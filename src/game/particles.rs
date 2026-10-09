use glam::{Vec3, Vec4};
use crate::assets::models::{MeshData, ModelBuilder};
use rand::Rng;

pub struct BulletTracer {
    pub start: Vec3,
    pub current: Vec3,
    pub direction: Vec3,
    pub speed: f32,
    pub distance_traveled: f32,
    pub max_distance: f32,
    pub alive: bool,
}

pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: Vec4,
    pub size: f32,
    pub life: f32,
    pub max_life: f32,
}

pub struct ParticleSystem {
    pub tracers: Vec<BulletTracer>,
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            tracers: Vec::new(),
            particles: Vec::new(),
        }
    }

    pub fn spawn_tracer(&mut self, origin: Vec3, dir: Vec3, max_dist: f32) {
        self.tracers.push(BulletTracer {
            start: origin,
            current: origin,
            direction: dir.normalize(),
            speed: 350.0,
            distance_traveled: 0.0,
            max_distance: max_dist,
            alive: true,
        });
    }

    pub fn spawn_sparks(&mut self, pos: Vec3, normal: Vec3, count: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let spread = Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(0.2..1.5),
                rng.gen_range(-1.0..1.0),
            ).normalize();
            let vel = (normal * 4.0 + spread * 6.0) * rng.gen_range(0.8..1.8);
            let color = Vec4::new(1.0, rng.gen_range(0.6..0.9), 0.2, 1.0);
            self.particles.push(Particle {
                position: pos,
                velocity: vel,
                color,
                size: rng.gen_range(0.04..0.08),
                life: rng.gen_range(0.2..0.45),
                max_life: 0.45,
            });
        }
    }

    pub fn spawn_blood(&mut self, pos: Vec3, hit_dir: Vec3, count: usize) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let spread = Vec3::new(
                rng.gen_range(-0.5..0.5),
                rng.gen_range(-0.3..0.8),
                rng.gen_range(-0.5..0.5),
            );
            let vel = (hit_dir * 3.0 + spread * 4.0) * rng.gen_range(0.7..1.5);
            let color = Vec4::new(0.75, 0.05, 0.05, 1.0);
            self.particles.push(Particle {
                position: pos,
                velocity: vel,
                color,
                size: rng.gen_range(0.06..0.12),
                life: rng.gen_range(0.3..0.6),
                max_life: 0.6,
            });
        }
    }

    pub fn spawn_explosion(&mut self, pos: Vec3) {
        let mut rng = rand::thread_rng();
        for _ in 0..80 {
            let dir = Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(0.1..1.5),
                rng.gen_range(-1.0..1.0),
            ).normalize();
            let speed = rng.gen_range(6.0..22.0);
            let is_fire = rng.gen_bool(0.7);
            let color = if is_fire {
                Vec4::new(1.0, rng.gen_range(0.2..0.7), 0.05, 1.0)
            } else {
                Vec4::new(0.25, 0.22, 0.20, 1.0) // smoke/debris
            };

            self.particles.push(Particle {
                position: pos + Vec3::new(0.0, 0.3, 0.0),
                velocity: dir * speed,
                color,
                size: rng.gen_range(0.2..0.55),
                life: rng.gen_range(0.6..1.4),
                max_life: 1.4,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Update tracers
        for t in &mut self.tracers {
            let step = t.speed * dt;
            t.current += t.direction * step;
            t.distance_traveled += step;
            if t.distance_traveled >= t.max_distance {
                t.alive = false;
            }
        }
        self.tracers.retain(|t| t.alive);

        // Update particles
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.velocity.y -= 16.0 * dt; // gravity
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn build_mesh(&self) -> MeshData {
        let mut b = ModelBuilder::new();

        // Build tracers
        for t in &self.tracers {
            let tracer_len = 2.5;
            let tail = t.current - t.direction * tracer_len;
            b.add_box(
                (t.current + tail) * 0.5,
                Vec3::new(0.03, 0.03, tracer_len * 0.5),
                Vec4::new(1.0, 0.85, 0.3, 1.0),
            );
        }

        // Build particles
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut col = p.color;
            col.w *= alpha;
            b.add_box(p.position, Vec3::splat(p.size), col);
        }

        b.build()
    }
}
