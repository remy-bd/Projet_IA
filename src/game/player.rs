use glam::Vec3;
use crate::audio::sounds::SoundEngine;
use crate::engine::camera::Camera;
use crate::game::bot::{BotManager, BotState};
use crate::game::map::{ApocalypseMap, AABB};
use crate::game::particles::ParticleSystem;
use crate::game::weapon::{Weapon, WeaponType};
use rand::Rng;

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub is_grounded: bool,
    pub is_sprinting: bool,
    pub is_crouching: bool,
    pub eye_height: f32,
    pub target_eye_height: f32,
    pub walk_bob_timer: f32,
    pub walk_bob_offset: Vec3,
    pub weapons: Vec<Weapon>,
    pub current_weapon_idx: usize,
    pub current_spawn_idx: usize,
    pub muzzle_light_timer: f32,
    pub muzzle_light_pos: Vec3,
    pub kills: u32,
    pub deaths: u32,
    pub hitmarker_timer: f32,
}

impl Player {
    pub fn new(spawn_idx: usize, spawn_pos: Vec3) -> Self {
        Self {
            position: spawn_pos,
            velocity: Vec3::ZERO,
            health: 100.0,
            max_health: 100.0,
            is_grounded: true,
            is_sprinting: false,
            is_crouching: false,
            eye_height: 1.75,
            target_eye_height: 1.75,
            walk_bob_timer: 0.0,
            walk_bob_offset: Vec3::ZERO,
            weapons: vec![
                Weapon::new_akm(),
                Weapon::new_m4a1(),
                Weapon::new_colt1911(),
            ],
            current_weapon_idx: 0,
            current_spawn_idx: spawn_idx,
            muzzle_light_timer: 0.0,
            muzzle_light_pos: Vec3::ZERO,
            kills: 0,
            deaths: 0,
            hitmarker_timer: 0.0,
        }
    }

    pub fn current_weapon(&self) -> &Weapon {
        &self.weapons[self.current_weapon_idx]
    }

    pub fn current_weapon_mut(&mut self) -> &mut Weapon {
        &mut self.weapons[self.current_weapon_idx]
    }

    pub fn switch_weapon(&mut self, idx: usize) {
        if idx < self.weapons.len() && idx != self.current_weapon_idx {
            self.current_weapon_idx = idx;
            let w = &mut self.weapons[self.current_weapon_idx];
            w.state = crate::game::weapon::WeaponState::Equipping(0.25);
            w.is_ads = false;
        }
    }

    pub fn respawn(&mut self, map: &ApocalypseMap) {
        let (idx, pos) = map.random_spawn(Some(self.current_spawn_idx));
        self.current_spawn_idx = idx;
        self.position = pos;
        self.velocity = Vec3::ZERO;
        self.health = 100.0;
        self.deaths += 1;
        for w in &mut self.weapons {
            w.ammo_mag = w.mag_capacity;
            w.state = crate::game::weapon::WeaponState::Idle;
            w.is_ads = false;
        }
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.position + Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.4, 0.9, 0.4))
    }

    pub fn camera_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, self.eye_height, 0.0)
    }

    pub fn shoot(
        &mut self,
        camera: &mut Camera,
        map: &mut ApocalypseMap,
        bot_mgr: &mut BotManager,
        particles: &mut ParticleSystem,
        sound: &SoundEngine,
    ) {
        let cur_idx = self.current_weapon_idx;
        let can_fire = self.weapons[cur_idx].can_fire();
        let ammo_mag = self.weapons[cur_idx].ammo_mag;
        let time_since_last_shot = self.weapons[cur_idx].time_since_last_shot;
        let weapon_type = self.weapons[cur_idx].weapon_type;
        let is_ads = self.weapons[cur_idx].is_ads;
        let recoil_v = self.weapons[cur_idx].recoil_vertical;
        let recoil_h = self.weapons[cur_idx].recoil_horizontal;
        let spread_val = self.weapons[cur_idx].spread_current;
        let base_damage = self.weapons[cur_idx].damage;

        if !can_fire {
            if ammo_mag == 0 && time_since_last_shot > 0.2 {
                sound.play_dry_fire();
                self.weapons[cur_idx].time_since_last_shot = 0.0;
            }
            return;
        }

        // Fire sound
        match weapon_type {
            WeaponType::AKM => sound.play_akm(),
            WeaponType::M4A1 => sound.play_m4a1(),
            WeaponType::Colt1911 => sound.play_colt(),
        }

        // Camera recoil kick
        let mut rng = rand::thread_rng();
        let recoil_p = recoil_v * (if is_ads { 0.55 } else { 1.0 });
        let recoil_y = rng.gen_range(-recoil_h..recoil_h);
        camera.add_recoil(recoil_p, recoil_y);
        camera.add_shake(0.02);

        // Raycast calculation
        let eye_pos = self.camera_position();
        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();

        let spread_offset = right * rng.gen_range(-spread_val..spread_val) + up * rng.gen_range(-spread_val..spread_val);
        let shoot_dir = (forward + spread_offset).normalize();

        // Muzzle flash
        self.muzzle_light_timer = 0.05;
        self.muzzle_light_pos = eye_pos + forward * 0.6 + right * 0.2 - up * 0.15;

        // Perform raycast against map, bots, and barrels
        let max_range = 150.0;
        let mut hit_distance = map.raycast(eye_pos, shoot_dir, max_range).unwrap_or(max_range);
        let mut hit_anything = false;

        // Check bots
        for bot in &mut bot_mgr.bots {
            if matches!(bot.state, BotState::Dead { .. }) {
                continue;
            }
            let bot_aabb = bot.aabb();
            if let Some(dist) = bot_aabb.intersects_ray(eye_pos, shoot_dir, hit_distance) {
                hit_distance = dist;
                let hit_pos = eye_pos + shoot_dir * dist;
                let is_headshot = hit_pos.y > (bot.position.y + 1.4);
                let damage_multiplier = if is_headshot { 2.2 } else { 1.0 };
                let damage = base_damage * damage_multiplier;

                particles.spawn_blood(hit_pos, shoot_dir, 10);
                self.hitmarker_timer = 0.15;

                if bot.take_damage(damage, sound) {
                    self.kills += 1;
                    bot_mgr.kills += 1;
                    // Loot ammo reward
                    for weapon in &mut self.weapons {
                        weapon.ammo_reserve = (weapon.ammo_reserve + weapon.mag_capacity).min(weapon.max_reserve);
                    }
                }
                hit_anything = true;
                break;
            }
        }

        // Check explosive barrels
        for barrel in &mut map.barrels {
            if barrel.exploded {
                continue;
            }
            if let Some(dist) = barrel.aabb.intersects_ray(eye_pos, shoot_dir, hit_distance) {
                hit_distance = dist;
                barrel.health -= base_damage;
                let hit_pos = eye_pos + shoot_dir * dist;
                particles.spawn_sparks(hit_pos, -shoot_dir, 8);

                if barrel.health <= 0.0 && !barrel.exploded {
                    barrel.exploded = true;
                    sound.play_explosion();
                    particles.spawn_explosion(barrel.position);
                    camera.add_shake(0.35);

                    // Deal AoE damage
                    let b_pos = barrel.position;
                    let p_dist = (self.position - b_pos).length();
                    if p_dist < 12.0 {
                        let aoe_dmg = (1.0 - p_dist / 12.0) * 85.0;
                        self.health = (self.health - aoe_dmg).max(0.0);
                        sound.play_hurt();
                    }

                    for bot in &mut bot_mgr.bots {
                        let bot_dist = (bot.position - b_pos).length();
                        if bot_dist < 12.0 {
                            let bot_dmg = (1.0 - bot_dist / 12.0) * 160.0;
                            if bot.take_damage(bot_dmg, sound) {
                                self.kills += 1;
                                bot_mgr.kills += 1;
                            }
                        }
                    }
                }
                hit_anything = true;
                break;
            }
        }

        if !hit_anything && hit_distance < max_range {
            let hit_point = eye_pos + shoot_dir * hit_distance;
            particles.spawn_sparks(hit_point, -shoot_dir, 6);
        }

        particles.spawn_tracer(self.muzzle_light_pos, shoot_dir, hit_distance);
        self.weapons[cur_idx].on_fire();
    }

    pub fn update(
        &mut self,
        dt: f32,
        move_input: Vec3,
        camera: &mut Camera,
        map: &ApocalypseMap,
        sound: &SoundEngine,
    ) {
        self.hitmarker_timer = (self.hitmarker_timer - dt).max(0.0);
        self.muzzle_light_timer = (self.muzzle_light_timer - dt).max(0.0);

        // Weapon update
        for w in &mut self.weapons {
            w.update(dt, sound);
        }

        // Crouch height transition
        self.target_eye_height = if self.is_crouching { 1.0 } else { 1.75 };
        self.eye_height += (self.target_eye_height - self.eye_height) * (12.0 * dt).min(1.0);

        // Movement Speed
        let base_speed = if self.is_sprinting && !self.is_crouching && !self.current_weapon().is_ads {
            8.2
        } else if self.is_crouching {
            3.0
        } else if self.current_weapon().is_ads {
            3.2
        } else {
            5.2
        };

        let fwd_yaw = camera.yaw;
        let forward_dir = Vec3::new(fwd_yaw.cos(), 0.0, fwd_yaw.sin()).normalize();
        let right_dir = Vec3::new(-fwd_yaw.sin(), 0.0, fwd_yaw.cos()).normalize();

        let desired_vel = (forward_dir * move_input.z + right_dir * move_input.x).normalize_or_zero() * base_speed;

        // Ground acceleration & friction
        let accel_rate = if self.is_grounded { 14.0 } else { 3.0 };
        self.velocity.x += (desired_vel.x - self.velocity.x) * (accel_rate * dt).min(1.0);
        self.velocity.z += (desired_vel.z - self.velocity.z) * (accel_rate * dt).min(1.0);

        // Gravity
        if !self.is_grounded {
            self.velocity.y -= 22.0 * dt;
        }

        // Horizontal Movement with Collision
        let delta_x = Vec3::new(self.velocity.x * dt, 0.0, 0.0);
        let test_pos_x = self.position + delta_x;
        let test_aabb_x = AABB::new(test_pos_x + Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.4, 0.85, 0.4));
        if !map.check_collision(&test_aabb_x) {
            self.position += delta_x;
        } else {
            self.velocity.x = 0.0;
        }

        let delta_z = Vec3::new(0.0, 0.0, self.velocity.z * dt);
        let test_pos_z = self.position + delta_z;
        let test_aabb_z = AABB::new(test_pos_z + Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.4, 0.85, 0.4));
        if !map.check_collision(&test_aabb_z) {
            self.position += delta_z;
        } else {
            self.velocity.z = 0.0;
        }

        // Vertical Movement with Ground Check
        self.position.y += self.velocity.y * dt;
        if self.position.y <= 0.0 {
            self.position.y = 0.0;
            self.velocity.y = 0.0;
            self.is_grounded = true;
        } else {
            self.is_grounded = false;
        }

        // Walk Bobbing Animation
        let horiz_speed = Vec3::new(self.velocity.x, 0.0, self.velocity.z).length();
        if self.is_grounded && horiz_speed > 0.5 {
            let bob_freq = if self.is_sprinting { 14.0 } else { 10.0 };
            self.walk_bob_timer += dt * bob_freq;
            let bob_y = (self.walk_bob_timer).sin() * 0.025;
            let bob_x = (self.walk_bob_timer * 0.5).cos() * 0.02;
            self.walk_bob_offset = Vec3::new(bob_x, bob_y, 0.0);

            // Footstep audio trigger
            if (self.walk_bob_timer % std::f32::consts::PI) < 0.15 {
                sound.play_footstep();
            }
        } else {
            self.walk_bob_offset *= (-dt * 10.0).exp();
        }

        // Update Camera FOV for ADS
        let target_fov = if self.current_weapon().is_ads { 50.0 } else { 75.0 };
        camera.fov += (target_fov - camera.fov) * (14.0 * dt).min(1.0);
        camera.position = self.camera_position();
        camera.update_decay(dt);

        // Auto respawn if health is 0
        if self.health <= 0.0 {
            self.respawn(map);
        }
    }

    pub fn jump(&mut self) {
        if self.is_grounded {
            self.velocity.y = 7.5;
            self.is_grounded = false;
        }
    }
}
