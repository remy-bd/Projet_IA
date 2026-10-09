use glam::{Vec3, Vec4};
use crate::assets::models::{MeshData, ModelBuilder};
use crate::engine::camera::Camera;
use crate::game::bot::BotManager;
use crate::game::player::Player;

pub struct HudRenderer {
    //
}

impl HudRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_hud_mesh(
        &self,
        player: &Player,
        bot_mgr: &BotManager,
        camera: &Camera,
        _aspect: f32,
    ) -> MeshData {
        let mut b = ModelBuilder::new();
        let weapon = player.current_weapon();

        // 1. Dynamic Crosshair (only in Hipfire or slight cross in ADS)
        let ch_alpha = (1.0 - weapon.ads_progress * 0.9).clamp(0.0, 1.0);
        if ch_alpha > 0.05 {
            let ch_spread = weapon.spread_current * 1.5;
            let ch_len = 0.025;
            let ch_thick = 0.0035;
            let ch_color = Vec4::new(1.0, 1.0, 1.0, 0.85 * ch_alpha);

            // Left tick
            b.add_box(
                Vec3::new(-ch_spread - ch_len * 0.5, 0.0, -1.0),
                Vec3::new(ch_len * 0.5, ch_thick, 0.001),
                ch_color,
            );
            // Right tick
            b.add_box(
                Vec3::new(ch_spread + ch_len * 0.5, 0.0, -1.0),
                Vec3::new(ch_len * 0.5, ch_thick, 0.001),
                ch_color,
            );
            // Top tick
            b.add_box(
                Vec3::new(0.0, ch_spread + ch_len * 0.5, -1.0),
                Vec3::new(ch_thick, ch_len * 0.5, 0.001),
                ch_color,
            );
            // Bottom tick
            b.add_box(
                Vec3::new(0.0, -ch_spread - ch_len * 0.5, -1.0),
                Vec3::new(ch_thick, ch_len * 0.5, 0.001),
                ch_color,
            );
            // Center dot
            b.add_box(
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(ch_thick, ch_thick, 0.001),
                Vec4::new(1.0, 0.3, 0.2, 0.9 * ch_alpha),
            );
        }

        // 2. Hitmarker (X shape when shooting an enemy)
        if player.hitmarker_timer > 0.01 {
            let hm_color = Vec4::new(1.0, 0.15, 0.15, (player.hitmarker_timer / 0.15).min(1.0));
            let hm_size = 0.022;
            let hm_thick = 0.004;
            // Diagonals
            b.add_box(Vec3::new(hm_size, hm_size, -0.99), Vec3::new(hm_thick, hm_thick, 0.001), hm_color);
            b.add_box(Vec3::new(-hm_size, hm_size, -0.99), Vec3::new(hm_thick, hm_thick, 0.001), hm_color);
            b.add_box(Vec3::new(hm_size, -hm_size, -0.99), Vec3::new(hm_thick, hm_thick, 0.001), hm_color);
            b.add_box(Vec3::new(-hm_size, -hm_size, -0.99), Vec3::new(hm_thick, hm_thick, 0.001), hm_color);
        }

        // 3. Health Bar (Bottom Left)
        let hp_ratio = (player.health / player.max_health).clamp(0.0, 1.0);
        let hp_bar_w = 0.35;
        let hp_bar_h = 0.035;
        let hp_pos_x = -0.65;
        let hp_pos_y = -0.75;

        // Background
        b.add_box(
            Vec3::new(hp_pos_x + hp_bar_w * 0.5, hp_pos_y, -1.0),
            Vec3::new(hp_bar_w * 0.5 + 0.008, hp_bar_h * 0.5 + 0.008, 0.001),
            Vec4::new(0.05, 0.05, 0.05, 0.8),
        );
        // Fill
        let hp_col = if hp_ratio > 0.5 {
            Vec4::new(0.2, 0.85, 0.3, 0.9)
        } else if hp_ratio > 0.25 {
            Vec4::new(0.9, 0.7, 0.1, 0.9)
        } else {
            Vec4::new(0.9, 0.15, 0.15, 0.9)
        };
        b.add_box(
            Vec3::new(hp_pos_x + (hp_bar_w * hp_ratio) * 0.5, hp_pos_y, -0.99),
            Vec3::new((hp_bar_w * hp_ratio) * 0.5, hp_bar_h * 0.5, 0.001),
            hp_col,
        );

        // 4. Ammo Bar / Visual Counter (Bottom Right)
        let ammo_ratio = (weapon.ammo_mag as f32 / weapon.mag_capacity as f32).clamp(0.0, 1.0);
        let ammo_bar_w = 0.35;
        let ammo_bar_h = 0.035;
        let ammo_pos_x = 0.30;
        let ammo_pos_y = -0.75;

        // Ammo BG
        b.add_box(
            Vec3::new(ammo_pos_x + ammo_bar_w * 0.5, ammo_pos_y, -1.0),
            Vec3::new(ammo_bar_w * 0.5 + 0.008, ammo_bar_h * 0.5 + 0.008, 0.001),
            Vec4::new(0.05, 0.05, 0.05, 0.8),
        );
        // Ammo Fill
        let ammo_fill_col = if ammo_ratio > 0.3 {
            Vec4::new(0.95, 0.75, 0.15, 0.9)
        } else {
            Vec4::new(0.9, 0.2, 0.1, 0.9)
        };
        b.add_box(
            Vec3::new(ammo_pos_x + (ammo_bar_w * ammo_ratio) * 0.5, ammo_pos_y, -0.99),
            Vec3::new((ammo_bar_w * ammo_ratio) * 0.5, ammo_bar_h * 0.5, 0.001),
            ammo_fill_col,
        );

        // Weapon Selection Icons (Keys 1, 2, 3 indicators)
        for i in 0..3 {
            let is_selected = i == player.current_weapon_idx;
            let icon_x = 0.32 + (i as f32) * 0.12;
            let icon_y = -0.68;
            let col = if is_selected {
                Vec4::new(1.0, 0.85, 0.2, 0.9)
            } else {
                Vec4::new(0.4, 0.4, 0.4, 0.5)
            };
            b.add_box(Vec3::new(icon_x, icon_y, -0.99), Vec3::new(0.045, 0.015, 0.001), col);
        }

        // 5. Radar / Compass (Top Center)
        let radar_y = 0.82;
        let compass_w = 0.50;
        b.add_box(
            Vec3::new(0.0, radar_y, -1.0),
            Vec3::new(compass_w * 0.5, 0.02, 0.001),
            Vec4::new(0.1, 0.1, 0.1, 0.7),
        );
        // Center tick (forward direction)
        b.add_box(
            Vec3::new(0.0, radar_y, -0.99),
            Vec3::new(0.004, 0.03, 0.001),
            Vec4::new(1.0, 0.3, 0.2, 0.95),
        );

        // Enemy blips on compass
        for bot in &bot_mgr.bots {
            if matches!(bot.state, crate::game::bot::BotState::Dead { .. }) {
                continue;
            }
            let to_bot = bot.position - player.position;
            let angle_to_bot = to_bot.z.atan2(to_bot.x);
            let mut diff = angle_to_bot - camera.yaw;
            while diff > std::f32::consts::PI { diff -= std::f32::consts::TAU; }
            while diff < -std::f32::consts::PI { diff += std::f32::consts::TAU; }

            if diff.abs() < 1.1 {
                let blip_x = (diff / 1.1) * (compass_w * 0.45);
                b.add_box(
                    Vec3::new(blip_x, radar_y, -0.98),
                    Vec3::new(0.008, 0.015, 0.001),
                    Vec4::new(1.0, 0.1, 0.1, 0.9),
                );
            }
        }

        // 6. Blood Vignette Screen Flash when low HP
        if player.health < 40.0 {
            let pulse = ((player.walk_bob_timer * 3.0).sin() * 0.5 + 0.5) * (1.0 - player.health / 40.0) * 0.4;
            let blood_vignette = Vec4::new(0.6, 0.0, 0.0, pulse);
            // Border edges
            b.add_box(Vec3::new(-0.95, 0.0, -0.9), Vec3::new(0.08, 1.0, 0.001), blood_vignette);
            b.add_box(Vec3::new(0.95, 0.0, -0.9), Vec3::new(0.08, 1.0, 0.001), blood_vignette);
            b.add_box(Vec3::new(0.0, -0.95, -0.9), Vec3::new(1.0, 0.08, 0.001), blood_vignette);
            b.add_box(Vec3::new(0.0, 0.95, -0.9), Vec3::new(1.0, 0.08, 0.001), blood_vignette);
        }

        b.build()
    }
}
