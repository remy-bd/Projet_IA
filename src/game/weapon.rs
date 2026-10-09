use glam::{Mat4, Quat, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    AKM,
    M4A1,
    Colt1911,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponState {
    Idle,
    Firing(f32), // time remaining in fire recoil animation
    Reloading {
        time_left: f32,
        total_time: f32,
        mag_out: bool,
        mag_inserted: bool,
        bolt_racked: bool,
    },
    Equipping(f32),
}

pub struct Weapon {
    pub weapon_type: WeaponType,
    pub name: &'static str,
    pub ammo_mag: u32,
    pub mag_capacity: u32,
    pub ammo_reserve: u32,
    pub max_reserve: u32,
    pub damage: f32,
    pub fire_rate: f32, // rounds per second
    pub auto_fire: bool,
    pub reload_duration: f32,
    pub recoil_vertical: f32,
    pub recoil_horizontal: f32,
    pub spread_base: f32,
    pub spread_current: f32,
    pub state: WeaponState,
    pub time_since_last_shot: f32,
    pub hip_offset: Vec3,
    pub ads_offset: Vec3,
    pub base_scale: Vec3,
    pub muzzle_local_pos: Vec3,
    pub is_ads: bool,
    pub ads_progress: f32, // 0.0 (Hip) to 1.0 (ADS)
    pub recoil_anim_pos: Vec3,
    pub recoil_anim_rot: Vec3,
}

impl Weapon {
    pub fn new_akm() -> Self {
        Self {
            weapon_type: WeaponType::AKM,
            name: "AKM 7.62x39mm",
            ammo_mag: 30,
            mag_capacity: 30,
            ammo_reserve: 120,
            max_reserve: 180,
            damage: 38.0,
            fire_rate: 10.0, // 600 RPM
            auto_fire: true,
            reload_duration: 2.4,
            recoil_vertical: 0.038,
            recoil_horizontal: 0.012,
            spread_base: 0.022,
            spread_current: 0.022,
            state: WeaponState::Idle,
            time_since_last_shot: 1.0,
            hip_offset: Vec3::new(0.24, -0.22, -0.42),
            ads_offset: Vec3::new(0.0, -0.155, -0.32),
            base_scale: Vec3::splat(1.1),
            muzzle_local_pos: Vec3::new(0.0, 0.015, 0.65),
            is_ads: false,
            ads_progress: 0.0,
            recoil_anim_pos: Vec3::ZERO,
            recoil_anim_rot: Vec3::ZERO,
        }
    }

    pub fn new_m4a1() -> Self {
        Self {
            weapon_type: WeaponType::M4A1,
            name: "M4A1 Carbine 5.56mm",
            ammo_mag: 30,
            mag_capacity: 30,
            ammo_reserve: 150,
            max_reserve: 210,
            damage: 29.0,
            fire_rate: 13.3, // 800 RPM
            auto_fire: true,
            reload_duration: 2.1,
            recoil_vertical: 0.026,
            recoil_horizontal: 0.008,
            spread_base: 0.014,
            spread_current: 0.014,
            state: WeaponState::Idle,
            time_since_last_shot: 1.0,
            hip_offset: Vec3::new(0.22, -0.21, -0.40),
            ads_offset: Vec3::new(0.0, -0.165, -0.30),
            base_scale: Vec3::splat(1.1),
            muzzle_local_pos: Vec3::new(0.0, 0.02, 0.60),
            is_ads: false,
            ads_progress: 0.0,
            recoil_anim_pos: Vec3::ZERO,
            recoil_anim_rot: Vec3::ZERO,
        }
    }

    pub fn new_colt1911() -> Self {
        Self {
            weapon_type: WeaponType::Colt1911,
            name: "Colt 1911 .45 ACP",
            ammo_mag: 7,
            mag_capacity: 7,
            ammo_reserve: 35,
            max_reserve: 56,
            damage: 54.0,
            fire_rate: 5.5, // Semi-auto max rate
            auto_fire: false,
            reload_duration: 1.6,
            recoil_vertical: 0.052,
            recoil_horizontal: 0.006,
            spread_base: 0.018,
            spread_current: 0.018,
            state: WeaponState::Idle,
            time_since_last_shot: 1.0,
            hip_offset: Vec3::new(0.20, -0.18, -0.35),
            ads_offset: Vec3::new(0.0, -0.138, -0.28),
            base_scale: Vec3::splat(1.1),
            muzzle_local_pos: Vec3::new(0.0, 0.035, 0.18),
            is_ads: false,
            ads_progress: 0.0,
            recoil_anim_pos: Vec3::ZERO,
            recoil_anim_rot: Vec3::ZERO,
        }
    }

    pub fn can_fire(&self) -> bool {
        let shot_interval = 1.0 / self.fire_rate;
        self.time_since_last_shot >= shot_interval
            && matches!(self.state, WeaponState::Idle | WeaponState::Firing(_))
            && self.ammo_mag > 0
    }

    pub fn start_reload(&mut self) -> bool {
        if self.ammo_mag < self.mag_capacity && self.ammo_reserve > 0 {
            if !matches!(self.state, WeaponState::Reloading { .. }) {
                self.state = WeaponState::Reloading {
                    time_left: self.reload_duration,
                    total_time: self.reload_duration,
                    mag_out: false,
                    mag_inserted: false,
                    bolt_racked: false,
                };
                return true;
            }
        }
        false
    }

    pub fn update(&mut self, dt: f32, sound: &crate::audio::sounds::SoundEngine) {
        self.time_since_last_shot += dt;

        // ADS transition interpolation
        let ads_target = if self.is_ads { 1.0 } else { 0.0 };
        self.ads_progress += (ads_target - self.ads_progress) * (14.0 * dt).min(1.0);

        // Spread recovery
        let target_spread = if self.is_ads {
            self.spread_base * 0.3
        } else {
            self.spread_base
        };
        self.spread_current += (target_spread - self.spread_current) * (10.0 * dt).min(1.0);

        // Recoil animation decay
        self.recoil_anim_pos *= (-dt * 20.0).exp();
        self.recoil_anim_rot *= (-dt * 22.0).exp();

        // State Machine update
        match self.state {
            WeaponState::Firing(ref mut t) => {
                *t -= dt;
                if *t <= 0.0 {
                    self.state = WeaponState::Idle;
                }
            }
            WeaponState::Reloading {
                ref mut time_left,
                total_time,
                ref mut mag_out,
                ref mut mag_inserted,
                ref mut bolt_racked,
            } => {
                *time_left -= dt;
                let progress = 1.0 - (*time_left / total_time);

                if progress > 0.08 && !*mag_out {
                    *mag_out = true;
                    sound.play_mag_out();
                }

                if progress > 0.45 && !*mag_inserted {
                    *mag_inserted = true;
                    sound.play_mag_in();
                }

                if progress > 0.80 && !*bolt_racked {
                    *bolt_racked = true;
                    sound.play_bolt_rack();
                }

                if *time_left <= 0.0 {
                    let needed = self.mag_capacity - self.ammo_mag;
                    let to_load = needed.min(self.ammo_reserve);
                    self.ammo_mag += to_load;
                    self.ammo_reserve -= to_load;
                    self.state = WeaponState::Idle;
                }
            }
            WeaponState::Equipping(ref mut t) => {
                *t -= dt;
                if *t <= 0.0 {
                    self.state = WeaponState::Idle;
                }
            }
            WeaponState::Idle => {}
        }
    }

    pub fn on_fire(&mut self) {
        if self.ammo_mag > 0 {
            self.ammo_mag -= 1;
            self.time_since_last_shot = 0.0;
            self.state = WeaponState::Firing(0.12);
            self.spread_current = (self.spread_current + 0.025).min(0.08);

            // Viewmodel recoil kick
            self.recoil_anim_pos += Vec3::new(0.0, 0.02, 0.06);
            self.recoil_anim_rot += Vec3::new(0.12, 0.02, 0.0);
        }
    }

    pub fn viewmodel_matrix(&self, camera_pos: Vec3, camera_yaw: f32, camera_pitch: f32, walk_bob: Vec3) -> Mat4 {
        let base_pos = self.hip_offset.lerp(self.ads_offset, self.ads_progress);
        let mut final_pos = base_pos + walk_bob * (1.0 - self.ads_progress * 0.8) + self.recoil_anim_pos;

        let mut reload_rot = Vec3::ZERO;
        if let WeaponState::Reloading { time_left, total_time, .. } = self.state {
            let p = 1.0 - (time_left / total_time);
            let sin_p = (p * PI).sin();
            final_pos.y -= sin_p * 0.15;
            final_pos.z += sin_p * 0.08;
            reload_rot = Vec3::new(-sin_p * 0.45, sin_p * 0.35, sin_p * 0.6);
        }

        let rot_y = Quat::from_rotation_y(camera_yaw);
        let rot_x = Quat::from_rotation_x(camera_pitch);
        let cam_rot = rot_y * rot_x;

        let anim_rot = Quat::from_euler(
            glam::EulerRot::YXZ,
            self.recoil_anim_rot.y + reload_rot.y,
            self.recoil_anim_rot.x + reload_rot.x,
            self.recoil_anim_rot.z + reload_rot.z,
        );

        let final_rot = cam_rot * anim_rot;
        let world_weapon_pos = camera_pos + cam_rot * final_pos;

        Mat4::from_scale_rotation_translation(self.base_scale, final_rot, world_weapon_pos)
    }
}
