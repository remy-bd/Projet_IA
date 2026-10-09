use glam::{Mat4, Quat, Vec3};
use crate::audio::sounds::SoundEngine;
use crate::game::map::{ApocalypseMap, AABB};
use crate::game::particles::ParticleSystem;
use crate::game::weapon::WeaponType;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BotState {
    Patrolling { target_pos: Vec3, wait_timer: f32 },
    Combat { attack_cooldown: f32, strafe_dir: f32, strafe_timer: f32 },
    Reloading { timer: f32 },
    Dead { fall_angle: f32, despawn_timer: f32 },
}

pub struct Bot {
    pub id: usize,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub max_health: f32,
    pub state: BotState,
    pub weapon_type: WeaponType,
    pub ammo_mag: u32,
    pub mag_capacity: u32,
    pub hit_flash: f32,
    pub time_since_shot: f32,
    pub muzzle_flash_timer: f32,
}

impl Bot {
    pub fn new(id: usize, spawn_pos: Vec3, weapon_type: WeaponType) -> Self {
        let (mag_cap, _) = match weapon_type {
            WeaponType::AKM => (30, 120),
            WeaponType::M4A1 => (30, 150),
            WeaponType::Colt1911 => (7, 35),
        };

        Self {
            id,
            position: spawn_pos,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            health: 100.0,
            max_health: 100.0,
            state: BotState::Patrolling {
                target_pos: spawn_pos + Vec3::new(5.0, 0.0, 5.0),
                wait_timer: 0.0,
            },
            weapon_type,
            ammo_mag: mag_cap,
            mag_capacity: mag_cap,
            hit_flash: 0.0,
            time_since_shot: 0.0,
            muzzle_flash_timer: 0.0,
        }
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.position + Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.45, 0.9, 0.45))
    }

    pub fn head_pos(&self) -> Vec3 {
        self.position + Vec3::new(0.0, 1.6, 0.0)
    }

    pub fn take_damage(&mut self, dmg: f32, sound: &SoundEngine) -> bool {
        if matches!(self.state, BotState::Dead { .. }) {
            return false;
        }

        self.health -= dmg;
        self.hit_flash = 1.0;
        sound.play_hitmarker();

        if self.health <= 0.0 {
            self.health = 0.0;
            self.state = BotState::Dead {
                fall_angle: 0.0,
                despawn_timer: 6.0,
            };
            sound.play_hurt();
            return true; // Just died
        }
        false
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        player_health: &mut f32,
        map: &ApocalypseMap,
        particles: &mut ParticleSystem,
        sound: &SoundEngine,
    ) {
        self.hit_flash = (self.hit_flash - dt * 5.0).max(0.0);
        self.time_since_shot += dt;
        self.muzzle_flash_timer = (self.muzzle_flash_timer - dt).max(0.0);

        let eye_pos = self.head_pos();
        let to_player = player_pos - eye_pos;
        let dist_to_player = to_player.length();
        let dir_to_player = to_player.normalize();

        // Raycast line of sight check
        let has_los = if dist_to_player < 65.0 {
            map.raycast(eye_pos, dir_to_player, dist_to_player - 0.5).is_none()
        } else {
            false
        };

        match self.state {
            BotState::Dead { ref mut fall_angle, ref mut despawn_timer } => {
                *fall_angle = (*fall_angle + dt * 4.0).min(std::f32::consts::FRAC_PI_2);
                *despawn_timer -= dt;
            }
            BotState::Reloading { ref mut timer } => {
                *timer -= dt;
                if *timer <= 0.0 {
                    self.ammo_mag = self.mag_capacity;
                    self.state = BotState::Combat {
                        attack_cooldown: 0.3,
                        strafe_dir: 1.0,
                        strafe_timer: 1.5,
                    };
                }
            }
            BotState::Patrolling { ref mut target_pos, ref mut wait_timer } => {
                if has_los {
                    self.state = BotState::Combat {
                        attack_cooldown: 0.4,
                        strafe_dir: 1.0,
                        strafe_timer: 1.0,
                    };
                    return;
                }

                let to_target = *target_pos - self.position;
                let dist_target = to_target.length();

                if dist_target < 1.5 {
                    *wait_timer -= dt;
                    if *wait_timer <= 0.0 {
                        let mut rng = rand::thread_rng();
                        let (_, new_spawn) = map.random_spawn(None);
                        *target_pos = new_spawn + Vec3::new(rng.gen_range(-8.0..8.0), 0.0, rng.gen_range(-8.0..8.0));
                        *wait_timer = rng.gen_range(2.0..4.0);
                    }
                } else {
                    let move_dir = Vec3::new(to_target.x, 0.0, to_target.z).normalize();
                    self.yaw = move_dir.z.atan2(move_dir.x);
                    let new_pos = self.position + move_dir * 3.5 * dt;
                    let test_aabb = AABB::new(new_pos + Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.4, 0.9, 0.4));
                    if !map.check_collision(&test_aabb) {
                        self.position = new_pos;
                    } else {
                        // Pick another target if stuck
                        *wait_timer = 0.0;
                    }
                }
            }
            BotState::Combat {
                ref mut attack_cooldown,
                ref mut strafe_dir,
                ref mut strafe_timer,
            } => {
                // Face player
                self.yaw = dir_to_player.z.atan2(dir_to_player.x);
                self.pitch = (to_player.y / dist_to_player).asin().clamp(-0.8, 0.8);

                // Strafe movement
                *strafe_timer -= dt;
                if *strafe_timer <= 0.0 {
                    let mut rng = rand::thread_rng();
                    *strafe_dir = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                    *strafe_timer = rng.gen_range(1.2..2.5);
                }

                let right_dir = Vec3::new(-dir_to_player.z, 0.0, dir_to_player.x).normalize();
                let strafe_step = right_dir * (*strafe_dir) * 2.8 * dt;
                let test_aabb = AABB::new(self.position + strafe_step + Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.4, 0.9, 0.4));
                if !map.check_collision(&test_aabb) {
                    self.position += strafe_step;
                }

                // If out of ammo, reload
                if self.ammo_mag == 0 {
                    self.state = BotState::Reloading { timer: 2.2 };
                    return;
                }

                // Attack logic
                *attack_cooldown -= dt;
                let (fire_rate_delay, bot_damage, accuracy) = match self.weapon_type {
                    WeaponType::AKM => (0.13, 14.0, 0.045),
                    WeaponType::M4A1 => (0.09, 10.0, 0.035),
                    WeaponType::Colt1911 => (0.35, 22.0, 0.030),
                };

                if *attack_cooldown <= 0.0 && has_los && *player_health > 0.0 {
                    let mut rng = rand::thread_rng();
                    self.ammo_mag -= 1;
                    *attack_cooldown = fire_rate_delay + rng.gen_range(0.0..0.04);
                    self.muzzle_flash_timer = 0.06;

                    // Play gunshot
                    match self.weapon_type {
                        WeaponType::AKM => sound.play_akm(),
                        WeaponType::M4A1 => sound.play_m4a1(),
                        WeaponType::Colt1911 => sound.play_colt(),
                    }

                    // Spread calculation
                    let spread_x = rng.gen_range(-accuracy..accuracy);
                    let spread_y = rng.gen_range(-accuracy..accuracy);
                    let spread_z = rng.gen_range(-accuracy..accuracy);
                    let shoot_dir = (dir_to_player + Vec3::new(spread_x, spread_y, spread_z)).normalize();

                    let shoot_origin = eye_pos + shoot_dir * 0.5;
                    particles.spawn_tracer(shoot_origin, shoot_dir, dist_to_player.min(80.0));

                    // Check hit on player
                    let hit_chance = 0.65;
                    if rng.gen_bool(hit_chance) && dist_to_player < 50.0 {
                        *player_health = (*player_health - bot_damage).max(0.0);
                        sound.play_hurt();
                        particles.spawn_blood(player_pos + Vec3::new(0.0, 0.9, 0.0), shoot_dir, 6);
                    }
                }
            }
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        let (fall_x, fall_y_offset) = if let BotState::Dead { fall_angle, .. } = self.state {
            (fall_angle, (1.0 - fall_angle.cos()) * -0.6)
        } else {
            (0.0, 0.0)
        };

        let rot = Quat::from_rotation_y(-self.yaw + std::f32::consts::FRAC_PI_2) * Quat::from_rotation_x(fall_x);
        let pos = self.position + Vec3::new(0.0, fall_y_offset, 0.0);
        Mat4::from_scale_rotation_translation(Vec3::splat(1.0), rot, pos)
    }
}

pub struct BotManager {
    pub bots: Vec<Bot>,
    pub max_bots: usize,
    pub kills: u32,
    pub next_id: usize,
}

impl BotManager {
    pub fn new(max_bots: usize, map: &ApocalypseMap, player_spawn_idx: usize) -> Self {
        let mut bots = Vec::new();
        let mut id = 0;
        let mut rng = rand::thread_rng();

        for i in 0..max_bots {
            let spawn_idx = (player_spawn_idx + 1 + i) % map.spawn_points.len();
            let pos = map.spawn_points[spawn_idx].position + Vec3::new(rng.gen_range(-3.0..3.0), 0.0, rng.gen_range(-3.0..3.0));
            let weapon = match rng.gen_range(0..3) {
                0 => WeaponType::AKM,
                1 => WeaponType::M4A1,
                _ => WeaponType::Colt1911,
            };
            bots.push(Bot::new(id, pos, weapon));
            id += 1;
        }

        Self {
            bots,
            max_bots,
            kills: 0,
            next_id: id,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        player_health: &mut f32,
        map: &ApocalypseMap,
        particles: &mut ParticleSystem,
        sound: &SoundEngine,
    ) {
        for bot in &mut self.bots {
            bot.update(dt, player_pos, player_health, map, particles, sound);
        }

        // Clean up despawned bots
        self.bots.retain(|b| {
            if let BotState::Dead { despawn_timer, .. } = b.state {
                despawn_timer > 0.0
            } else {
                true
            }
        });

        // Respawn new bots up to max_bots
        let mut rng = rand::thread_rng();
        while self.bots.len() < self.max_bots {
            let (_, spawn_pos) = map.random_spawn(None);
            let offset = Vec3::new(rng.gen_range(-4.0..4.0), 0.0, rng.gen_range(-4.0..4.0));
            let weapon = match rng.gen_range(0..3) {
                0 => WeaponType::AKM,
                1 => WeaponType::M4A1,
                _ => WeaponType::Colt1911,
            };
            self.bots.push(Bot::new(self.next_id, spawn_pos + offset, weapon));
            self.next_id += 1;
        }
    }
}
