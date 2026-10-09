mod assets;
mod audio;
mod engine;
mod game;

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3};
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, WindowBuilder};

use crate::assets::models::{
    create_akm_mesh, create_barrel_mesh, create_bot_mesh, create_colt1911_mesh, create_m4a1_mesh,
    ensure_asset_obj_files, load_obj_or_fallback,
};
use crate::audio::sounds::SoundEngine;
use crate::engine::camera::Camera;
use crate::engine::mesh::GpuMesh;
use crate::engine::renderer::{UniformData, VulkanRenderer};
use crate::game::bot::{BotManager, BotState};
use crate::game::hud::HudRenderer;
use crate::game::map::ApocalypseMap;
use crate::game::particles::ParticleSystem;
use crate::game::player::Player;
use crate::game::weapon::WeaponType;

fn main() {
    env_logger::init();
    log::info!("Starting Apocalypse 3D Vulkan FPS Engine...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("APOCALYPSE 3D - Vulkan FPS Engine (Rust)")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let _ = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
    window.set_cursor_visible(false);

    // Initialize Vulkan Renderer
    let mut renderer = pollster::block_on(VulkanRenderer::new(window.clone()));

    // Prepare Models & OBJ files
    let models_dir = Path::new("assets").join("models");
    ensure_asset_obj_files(&models_dir);

    let akm_mesh_data = load_obj_or_fallback(&models_dir.join("akm.obj"), create_akm_mesh);
    let m4a1_mesh_data = load_obj_or_fallback(&models_dir.join("m4a1.obj"), create_m4a1_mesh);
    let colt_mesh_data = load_obj_or_fallback(&models_dir.join("colt1911.obj"), create_colt1911_mesh);
    let bot_mesh_data = load_obj_or_fallback(&models_dir.join("enemy_bot.obj"), create_bot_mesh);
    let barrel_mesh_data = load_obj_or_fallback(&models_dir.join("barrel.obj"), create_barrel_mesh);

    let akm_gpu = GpuMesh::from_mesh_data(&renderer.device, &akm_mesh_data);
    let m4a1_gpu = GpuMesh::from_mesh_data(&renderer.device, &m4a1_mesh_data);
    let colt_gpu = GpuMesh::from_mesh_data(&renderer.device, &colt_mesh_data);
    let bot_gpu = GpuMesh::from_mesh_data(&renderer.device, &bot_mesh_data);
    let barrel_gpu = GpuMesh::from_mesh_data(&renderer.device, &barrel_mesh_data);

    // Audio Engine
    let sound_engine = SoundEngine::new();

    // Map & Spawns
    let mut map = ApocalypseMap::new();
    let map_gpu = GpuMesh::from_mesh_data(&renderer.device, &map.mesh);

    // Player initial spawn
    let (player_spawn_idx, player_spawn_pos) = map.random_spawn(None);
    let mut player = Player::new(player_spawn_idx, player_spawn_pos);

    // Camera
    let aspect = renderer.config.width as f32 / renderer.config.height as f32;
    let mut camera = Camera::new(player.camera_position(), aspect);

    // Bots (Spawn 6 active bots on the apocalyptic map)
    let mut bot_manager = BotManager::new(6, &map, player_spawn_idx);

    // Particles & HUD
    let mut particle_system = ParticleSystem::new();
    let hud = HudRenderer::new();

    // Input States
    let mut key_w = false;
    let mut key_s = false;
    let mut key_a = false;
    let mut key_d = false;
    let mut key_shift = false;
    let mut key_ctrl = false;
    let mut mouse_left_held = false;
    let mut mouse_right_held = false;
    let mut mouse_captured = true;

    let mut last_time = Instant::now();
    let mut fps_timer = Instant::now();
    let mut frame_count = 0;

    log::info!("Game initialization complete. Entering main render loop...");

    event_loop.set_control_flow(ControlFlow::Poll);

    let _ = event_loop.run(move |event, target| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                ..
            } => {
                if mouse_captured {
                    let sensitivity = 0.0022;
                    camera.yaw += (dx as f32) * sensitivity;
                    camera.pitch = (camera.pitch - (dy as f32) * sensitivity).clamp(-1.52, 1.52);
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta },
                ..
            } => {
                let scroll_val = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                if scroll_val > 0.1 {
                    let next_idx = (player.current_weapon_idx + 1) % player.weapons.len();
                    player.switch_weapon(next_idx);
                } else if scroll_val < -0.1 {
                    let prev_idx = if player.current_weapon_idx == 0 {
                        player.weapons.len() - 1
                    } else {
                        player.current_weapon_idx - 1
                    };
                    player.switch_weapon(prev_idx);
                }
            }
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(physical_size);
                    camera.aspect = renderer.config.width as f32 / renderer.config.height as f32;
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let is_pressed = state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => {
                            mouse_left_held = is_pressed;
                            if is_pressed && !mouse_captured {
                                mouse_captured = true;
                                let _ = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
                                window.set_cursor_visible(false);
                            }
                        }
                        MouseButton::Right => {
                            mouse_right_held = is_pressed;
                        }
                        _ => {}
                    }
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    let is_down = state == ElementState::Pressed;
                    match key {
                        // ZQSD / WASD support
                        KeyCode::KeyW | KeyCode::KeyZ => key_w = is_down,
                        KeyCode::KeyS => key_s = is_down,
                        KeyCode::KeyA | KeyCode::KeyQ => key_a = is_down,
                        KeyCode::KeyD => key_d = is_down,
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => key_shift = is_down,
                        KeyCode::ControlLeft | KeyCode::ControlRight => key_ctrl = is_down,
                        KeyCode::Space => {
                            if is_down {
                                player.jump();
                            }
                        }
                        KeyCode::KeyR => {
                            if is_down {
                                player.current_weapon_mut().start_reload();
                            }
                        }
                        KeyCode::Digit1 => {
                            if is_down {
                                player.switch_weapon(0);
                            }
                        }
                        KeyCode::Digit2 => {
                            if is_down {
                                player.switch_weapon(1);
                            }
                        }
                        KeyCode::Digit3 => {
                            if is_down {
                                player.switch_weapon(2);
                            }
                        }
                        KeyCode::Escape => {
                            if is_down {
                                mouse_captured = !mouse_captured;
                                if mouse_captured {
                                    let _ = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
                                    window.set_cursor_visible(false);
                                } else {
                                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                                    window.set_cursor_visible(true);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let dt = (now - last_time).as_secs_f32().min(0.1);
                    last_time = now;

                    frame_count += 1;
                    if fps_timer.elapsed().as_secs_f32() >= 0.5 {
                        let fps = (frame_count as f32 / fps_timer.elapsed().as_secs_f32()) as u32;
                        let cur_w = player.current_weapon();
                        let title = format!(
                            "APOCALYPSE 3D [Vulkan] | FPS: {} | Kills: {} | Bots: {}/{} | HP: {:.0}/100 | {} [{} / {}] | Spawn: {}",
                            fps,
                            player.kills,
                            bot_manager.bots.iter().filter(|b| !matches!(b.state, BotState::Dead { .. })).count(),
                            bot_manager.max_bots,
                            player.health,
                            cur_w.name,
                            cur_w.ammo_mag,
                            cur_w.ammo_reserve,
                            map.spawn_points[player.current_spawn_idx].name
                        );
                        window.set_title(&title);
                        fps_timer = Instant::now();
                        frame_count = 0;
                    }

                    // Process Input
                    let mut move_dir = Vec3::ZERO;
                    if key_w { move_dir.z += 1.0; }
                    if key_s { move_dir.z -= 1.0; }
                    if key_d { move_dir.x += 1.0; }
                    if key_a { move_dir.x -= 1.0; }

                    player.is_sprinting = key_shift && move_dir.z > 0.0;
                    player.is_crouching = key_ctrl;
                    player.current_weapon_mut().is_ads = mouse_right_held;

                    // Continuous or semi-auto shooting
                    if mouse_left_held && mouse_captured {
                        let is_auto = player.current_weapon().auto_fire;
                        if is_auto || player.current_weapon().can_fire() {
                            player.shoot(&mut camera, &mut map, &mut bot_manager, &mut particle_system, &sound_engine);
                            if !is_auto {
                                mouse_left_held = false; // require release for semi-auto
                            }
                        }
                    }

                    // Update Game Systems
                    player.update(dt, move_dir, &mut camera, &map, &sound_engine);
                    bot_manager.update(dt, player.camera_position(), &mut player.health, &map, &mut particle_system, &sound_engine);
                    particle_system.update(dt);

                    // Build dynamic meshes before render pass
                    let particles_mesh = particle_system.build_mesh();
                    let maybe_part_gpu = if !particles_mesh.vertices.is_empty() {
                        Some(GpuMesh::from_mesh_data(&renderer.device, &particles_mesh))
                    } else {
                        None
                    };

                    let hud_mesh = hud.build_hud_mesh(&player, &bot_manager, &camera, camera.aspect);
                    let maybe_hud_gpu = if !hud_mesh.vertices.is_empty() {
                        Some(GpuMesh::from_mesh_data(&renderer.device, &hud_mesh))
                    } else {
                        None
                    };

                    // Render Frame with Vulkan
                    let output = match renderer.surface.get_current_texture() {
                        Ok(texture) => texture,
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(renderer.size);
                            return;
                        }
                        Err(e) => {
                            log::error!("Surface error: {:?}", e);
                            return;
                        }
                    };

                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                    let vp_matrix = camera.view_projection_matrix();
                    let apocalyptic_fog_col = [0.24, 0.16, 0.12]; // Orange / Ashen smoky sky

                    // Render Pass
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Main Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.18,
                                        g: 0.13,
                                        b: 0.10,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &renderer.depth_texture_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        render_pass.set_pipeline(&renderer.render_pipeline);
                        render_pass.set_bind_group(0, &renderer.uniform_bind_group, &[]);

                        let (muzzle_pos, muzzle_intensity) = if player.muzzle_light_timer > 0.01 {
                            (player.muzzle_light_pos, 1.8)
                        } else {
                            (Vec3::ZERO, 0.0)
                        };

                        let base_uniforms = UniformData {
                            view_proj: vp_matrix.to_cols_array(),
                            model: Mat4::IDENTITY.to_cols_array(),
                            camera_pos: camera.position.to_array(),
                            pad0: 0.0,
                            sun_dir: Vec3::new(0.6, -1.0, 0.4).normalize().to_array(),
                            pad1: 0.0,
                            sun_color: [1.0, 0.75, 0.5], // Warm apocalyptic sun
                            ambient_intensity: 1.0,
                            fog_color: apocalyptic_fog_col,
                            fog_density: 0.018,
                            muzzle_pos: muzzle_pos.to_array(),
                            muzzle_intensity,
                            muzzle_color: [1.0, 0.7, 0.2],
                            hit_flash: 0.0,
                        };

                        // 1. Draw Map Mesh
                        renderer.update_uniforms(&base_uniforms);
                        render_pass.set_vertex_buffer(0, map_gpu.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(map_gpu.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..map_gpu.num_elements, 0, 0..1);

                        // 2. Draw Explosive Barrels
                        for barrel in &map.barrels {
                            if !barrel.exploded {
                                let mut barrel_u = base_uniforms;
                                barrel_u.model = Mat4::from_translation(barrel.position).to_cols_array();
                                renderer.update_uniforms(&barrel_u);
                                render_pass.set_vertex_buffer(0, barrel_gpu.vertex_buffer.slice(..));
                                render_pass.set_index_buffer(barrel_gpu.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                render_pass.draw_indexed(0..barrel_gpu.num_elements, 0, 0..1);
                            }
                        }

                        // 3. Draw Bots
                        for bot in &bot_manager.bots {
                            let mut bot_u = base_uniforms;
                            bot_u.model = bot.model_matrix().to_cols_array();
                            bot_u.hit_flash = bot.hit_flash;
                            renderer.update_uniforms(&bot_u);

                            render_pass.set_vertex_buffer(0, bot_gpu.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(bot_gpu.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..bot_gpu.num_elements, 0, 0..1);
                        }

                        // 4. Draw Particles & Tracers
                        if let Some(ref part_gpu) = maybe_part_gpu {
                            let mut part_u = base_uniforms;
                            part_u.model = Mat4::IDENTITY.to_cols_array();
                            renderer.update_uniforms(&part_u);

                            render_pass.set_vertex_buffer(0, part_gpu.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(part_gpu.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..part_gpu.num_elements, 0, 0..1);
                        }

                        // 5. Draw Viewmodel Weapon
                        let cur_weapon = player.current_weapon();
                        let weapon_model_mat = cur_weapon.viewmodel_matrix(
                            camera.position,
                            camera.yaw,
                            camera.pitch + camera.recoil_pitch,
                            player.walk_bob_offset,
                        );
                        let mut weapon_u = base_uniforms;
                        weapon_u.model = weapon_model_mat.to_cols_array();
                        weapon_u.fog_density = 0.0; // Don't fog viewmodel weapon
                        renderer.update_uniforms(&weapon_u);

                        let active_weapon_gpu = match cur_weapon.weapon_type {
                            WeaponType::AKM => &akm_gpu,
                            WeaponType::M4A1 => &m4a1_gpu,
                            WeaponType::Colt1911 => &colt_gpu,
                        };
                        render_pass.set_vertex_buffer(0, active_weapon_gpu.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(active_weapon_gpu.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..active_weapon_gpu.num_elements, 0, 0..1);

                        // 6. Draw 2D Tactical HUD (Crosshair, HP, Ammo, Compass Radar)
                        if let Some(ref hud_gpu) = maybe_hud_gpu {
                            let hud_ortho = Mat4::orthographic_rh(-camera.aspect, camera.aspect, -1.0, 1.0, 0.1, 10.0);
                            let mut hud_u = base_uniforms;
                            hud_u.view_proj = hud_ortho.to_cols_array();
                            hud_u.model = Mat4::IDENTITY.to_cols_array();
                            hud_u.fog_density = 0.0;
                            hud_u.ambient_intensity = 1.0;
                            renderer.update_uniforms(&hud_u);

                            render_pass.set_vertex_buffer(0, hud_gpu.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(hud_gpu.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..hud_gpu.num_elements, 0, 0..1);
                        }
                    }

                    renderer.queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
