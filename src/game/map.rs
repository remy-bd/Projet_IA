use glam::{Vec3, Vec4};
use crate::assets::models::{MeshData, ModelBuilder};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(center: Vec3, half_size: Vec3) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn intersects_ray(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<f32> {
        let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
        let t1 = (self.min.x - origin.x) * inv_dir.x;
        let t2 = (self.max.x - origin.x) * inv_dir.x;
        let t3 = (self.min.y - origin.y) * inv_dir.y;
        let t4 = (self.max.y - origin.y) * inv_dir.y;
        let t5 = (self.min.z - origin.z) * inv_dir.z;
        let t6 = (self.max.z - origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax || tmin > max_dist {
            None
        } else {
            Some(tmin.max(0.0))
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExplosiveBarrel {
    pub id: usize,
    pub position: Vec3,
    pub health: f32,
    pub exploded: bool,
    pub aabb: AABB,
}

pub struct SpawnPoint {
    pub name: &'static str,
    pub position: Vec3,
}

pub struct ApocalypseMap {
    pub colliders: Vec<AABB>,
    pub barrels: Vec<ExplosiveBarrel>,
    pub spawn_points: [SpawnPoint; 6],
    pub mesh: MeshData,
}

impl ApocalypseMap {
    pub fn new() -> Self {
        let mut b = ModelBuilder::new();
        let mut colliders = Vec::new();
        let mut barrels = Vec::new();

        let spawns = [
            SpawnPoint { name: "NW Ruined Factory", position: Vec3::new(-35.0, 1.0, -35.0) },
            SpawnPoint { name: "NE Overpass Debris", position: Vec3::new(35.0, 1.0, -35.0) },
            SpawnPoint { name: "Central Plaza Crater", position: Vec3::new(0.0, 1.0, 0.0) },
            SpawnPoint { name: "SW Container Yard", position: Vec3::new(-35.0, 1.0, 35.0) },
            SpawnPoint { name: "SE Military Outpost", position: Vec3::new(35.0, 1.0, 35.0) },
            SpawnPoint { name: "North Bunker Exit", position: Vec3::new(0.0, 1.0, -36.0) },
        ];

        let ground_color = Vec4::new(0.18, 0.16, 0.15, 1.0);
        let road_color = Vec4::new(0.12, 0.11, 0.11, 1.0);
        let concrete_wall = Vec4::new(0.32, 0.30, 0.28, 1.0);
        let brick_ruin = Vec4::new(0.42, 0.25, 0.20, 1.0);
        let rusty_metal = Vec4::new(0.38, 0.22, 0.14, 1.0);
        let sandbag_color = Vec4::new(0.45, 0.40, 0.28, 1.0);
        let steel_container_blue = Vec4::new(0.15, 0.25, 0.38, 1.0);
        let steel_container_rust = Vec4::new(0.45, 0.22, 0.12, 1.0);
        let car_body_wreck = Vec4::new(0.20, 0.20, 0.22, 1.0);

        // 1. Terrain Ground Base (140m x 140m)
        b.add_box(Vec3::new(0.0, -0.5, 0.0), Vec3::new(70.0, 0.5, 70.0), ground_color);

        // Asphalt roads crisscrossing the map
        b.add_box(Vec3::new(0.0, 0.02, 0.0), Vec3::new(7.0, 0.02, 68.0), road_color);
        b.add_box(Vec3::new(0.0, 0.02, 0.0), Vec3::new(68.0, 0.02, 7.0), road_color);

        // 2. Perimeter Concrete Blast Walls (Bordures de la map)
        let wall_h = 4.0;
        let wall_t = 1.0;
        let p_dist = 65.0;

        // North wall
        b.add_box(Vec3::new(0.0, wall_h / 2.0, -p_dist), Vec3::new(65.0, wall_h / 2.0, wall_t), concrete_wall);
        colliders.push(AABB::new(Vec3::new(0.0, wall_h / 2.0, -p_dist), Vec3::new(65.0, wall_h / 2.0, wall_t)));
        // South wall
        b.add_box(Vec3::new(0.0, wall_h / 2.0, p_dist), Vec3::new(65.0, wall_h / 2.0, wall_t), concrete_wall);
        colliders.push(AABB::new(Vec3::new(0.0, wall_h / 2.0, p_dist), Vec3::new(65.0, wall_h / 2.0, wall_t)));
        // West wall
        b.add_box(Vec3::new(-p_dist, wall_h / 2.0, 0.0), Vec3::new(wall_t, wall_h / 2.0, 65.0), concrete_wall);
        colliders.push(AABB::new(Vec3::new(-p_dist, wall_h / 2.0, 0.0), Vec3::new(wall_t, wall_h / 2.0, 65.0)));
        // East wall
        b.add_box(Vec3::new(p_dist, wall_h / 2.0, 0.0), Vec3::new(wall_t, wall_h / 2.0, 65.0), concrete_wall);
        colliders.push(AABB::new(Vec3::new(p_dist, wall_h / 2.0, 0.0), Vec3::new(wall_t, wall_h / 2.0, 65.0)));

        // Helper macro for adding collidable buildings/boxes
        let add_building = |b: &mut ModelBuilder, colliders: &mut Vec<AABB>, pos: Vec3, size: Vec3, col: Vec4| {
            b.add_box(pos, size, col);
            colliders.push(AABB::new(pos, size));
        };

        // 3. Ruined Multi-story Buildings & Structures
        // NW Warehouse ruins
        add_building(&mut b, &mut colliders, Vec3::new(-22.0, 4.0, -25.0), Vec3::new(8.0, 4.0, 10.0), brick_ruin);
        add_building(&mut b, &mut colliders, Vec3::new(-22.0, 9.0, -28.0), Vec3::new(7.0, 1.5, 6.0), concrete_wall);

        // NE Broken Complex
        add_building(&mut b, &mut colliders, Vec3::new(24.0, 4.5, -24.0), Vec3::new(9.0, 4.5, 8.0), concrete_wall);
        add_building(&mut b, &mut colliders, Vec3::new(26.0, 10.0, -22.0), Vec3::new(5.0, 2.0, 5.0), brick_ruin);

        // SE Military Fortress Outpost
        add_building(&mut b, &mut colliders, Vec3::new(25.0, 3.5, 25.0), Vec3::new(10.0, 3.5, 9.0), concrete_wall);
        add_building(&mut b, &mut colliders, Vec3::new(25.0, 8.0, 25.0), Vec3::new(2.5, 3.0, 2.5), rusty_metal); // Watchtower post

        // SW Industrial Hangar
        add_building(&mut b, &mut colliders, Vec3::new(-25.0, 3.5, 22.0), Vec3::new(9.0, 3.5, 11.0), brick_ruin);

        // 4. Shipping Container Clusters (Labyrinthe tactique)
        let containers = [
            (Vec3::new(-12.0, 1.4, -8.0), steel_container_blue, 0.0),
            (Vec3::new(-12.0, 1.4, -2.5), steel_container_rust, 0.0),
            (Vec3::new(-12.0, 4.2, -5.0), steel_container_blue, 0.0),
            (Vec3::new(14.0, 1.4, 8.0), steel_container_rust, 0.0),
            (Vec3::new(14.0, 1.4, 14.0), steel_container_blue, 0.0),
            (Vec3::new(-35.0, 1.4, 18.0), steel_container_blue, 0.0),
            (Vec3::new(-42.0, 1.4, 18.0), steel_container_rust, 0.0),
            (Vec3::new(38.0, 1.4, -18.0), steel_container_rust, 0.0),
        ];
        for (pos, col, _) in containers {
            add_building(&mut b, &mut colliders, pos, Vec3::new(2.8, 1.4, 1.4), col);
        }

        // 5. Tactical Barricades, Sandbags & Roadblocks
        let barricades = [
            Vec3::new(0.0, 0.6, 12.0),
            Vec3::new(0.0, 0.6, -12.0),
            Vec3::new(12.0, 0.6, 0.0),
            Vec3::new(-12.0, 0.6, 0.0),
            Vec3::new(8.0, 0.6, 8.0),
            Vec3::new(-8.0, 0.6, -8.0),
            Vec3::new(-18.0, 0.6, 32.0),
            Vec3::new(18.0, 0.6, -32.0),
        ];
        for pos in barricades {
            add_building(&mut b, &mut colliders, pos, Vec3::new(2.2, 0.6, 0.4), sandbag_color);
        }

        // 6. Car & Truck Wrecks (Voitures calcinées)
        let car_positions = [
            Vec3::new(-4.0, 0.8, 18.0),
            Vec3::new(5.0, 0.8, -19.0),
            Vec3::new(18.0, 0.8, -4.0),
            Vec3::new(-20.0, 0.8, 6.0),
            Vec3::new(-30.0, 0.8, -12.0),
            Vec3::new(32.0, 0.8, 12.0),
        ];
        for pos in car_positions {
            add_building(&mut b, &mut colliders, pos, Vec3::new(2.4, 0.7, 1.2), car_body_wreck);
            // Cabine de voiture
            b.add_box(pos + Vec3::new(0.0, 0.7, 0.0), Vec3::new(1.2, 0.5, 1.0), rusty_metal);
        }

        // 7. Central Plaza Monument Obelisk / Broken Tower
        add_building(&mut b, &mut colliders, Vec3::new(0.0, 3.5, 0.0), Vec3::new(1.8, 3.5, 1.8), concrete_wall);
        add_building(&mut b, &mut colliders, Vec3::new(0.0, 7.5, 0.0), Vec3::new(1.2, 1.8, 1.2), rusty_metal);

        // 8. 14 Explosive Red Barrels placed strategically
        let barrel_coords = [
            Vec3::new(-15.0, 0.5, -9.0),
            Vec3::new(-10.0, 0.5, -9.0),
            Vec3::new(16.0, 0.5, 6.0),
            Vec3::new(16.0, 0.5, 16.0),
            Vec3::new(-2.0, 0.5, 14.0),
            Vec3::new(2.0, 0.5, -14.0),
            Vec3::new(-28.0, 0.5, -18.0),
            Vec3::new(28.0, 0.5, -18.0),
            Vec3::new(-20.0, 0.5, 18.0),
            Vec3::new(20.0, 0.5, 20.0),
            Vec3::new(-32.0, 0.5, 30.0),
            Vec3::new(32.0, 0.5, 30.0),
            Vec3::new(-4.0, 0.5, -28.0),
            Vec3::new(4.0, 0.5, 28.0),
        ];

        for (i, pos) in barrel_coords.iter().enumerate() {
            let aabb = AABB::new(*pos, Vec3::new(0.4, 0.5, 0.4));
            colliders.push(aabb);
            barrels.push(ExplosiveBarrel {
                id: i,
                position: *pos,
                health: 40.0,
                exploded: false,
                aabb,
            });
        }

        Self {
            colliders,
            barrels,
            spawn_points: spawns,
            mesh: b.build(),
        }
    }

    pub fn random_spawn(&self, exclude_idx: Option<usize>) -> (usize, Vec3) {
        let mut rng = rand::thread_rng();
        loop {
            let idx = rng.gen_range(0..self.spawn_points.len());
            if Some(idx) != exclude_idx {
                return (idx, self.spawn_points[idx].position);
            }
        }
    }

    pub fn check_collision(&self, aabb: &AABB) -> bool {
        for c in &self.colliders {
            if c.intersects(aabb) {
                return true;
            }
        }
        false
    }

    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<f32> {
        let mut closest = max_dist;
        let mut hit = false;
        for c in &self.colliders {
            if let Some(d) = c.intersects_ray(origin, dir, closest) {
                closest = d;
                hit = true;
            }
        }
        if hit { Some(closest) } else { None }
    }
}
