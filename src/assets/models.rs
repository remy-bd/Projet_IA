use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    pub const fn new(pos: [f32; 3], norm: [f32; 3], col: [f32; 4], uv: [f32; 2]) -> Self {
        Self {
            position: pos,
            normal: norm,
            color: col,
            uv,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn export_to_obj_file(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        writeln!(file, "# Generated OBJ File for Apocalypse Vulkan FPS")?;
        
        for v in &self.vertices {
            writeln!(file, "v {:.5} {:.5} {:.5} {:.3} {:.3} {:.3}", 
                v.position[0], v.position[1], v.position[2],
                v.color[0], v.color[1], v.color[2]
            )?;
        }
        for v in &self.vertices {
            writeln!(file, "vn {:.4} {:.4} {:.4}", v.normal[0], v.normal[1], v.normal[2])?;
        }
        for v in &self.vertices {
            writeln!(file, "vt {:.4} {:.4}", v.uv[0], v.uv[1])?;
        }
        for chunk in self.indices.chunks(3) {
            if chunk.len() == 3 {
                let (i0, i1, i2) = (chunk[0] + 1, chunk[1] + 1, chunk[2] + 1);
                writeln!(file, "f {0}/{0}/{0} {1}/{1}/{1} {2}/{2}/{2}", i0, i1, i2)?;
            }
        }
        Ok(())
    }
}

pub struct ModelBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_box(
        &mut self,
        center: Vec3,
        half_size: Vec3,
        color: Vec4,
    ) {
        let base_idx = self.vertices.len() as u32;

        let p = [
            center + Vec3::new(-half_size.x, -half_size.y,  half_size.z), // 0 front-bot-left
            center + Vec3::new( half_size.x, -half_size.y,  half_size.z), // 1 front-bot-right
            center + Vec3::new( half_size.x,  half_size.y,  half_size.z), // 2 front-top-right
            center + Vec3::new(-half_size.x,  half_size.y,  half_size.z), // 3 front-top-left
            center + Vec3::new(-half_size.x, -half_size.y, -half_size.z), // 4 back-bot-left
            center + Vec3::new( half_size.x, -half_size.y, -half_size.z), // 5 back-bot-right
            center + Vec3::new( half_size.x,  half_size.y, -half_size.z), // 6 back-top-right
            center + Vec3::new(-half_size.x,  half_size.y, -half_size.z), // 7 back-top-left
        ];

        let faces = [
            ([0, 1, 2, 3], Vec3::new(0.0, 0.0, 1.0)),   // Front
            ([5, 4, 7, 6], Vec3::new(0.0, 0.0, -1.0)),  // Back
            ([4, 0, 3, 7], Vec3::new(-1.0, 0.0, 0.0)),  // Left
            ([1, 5, 6, 2], Vec3::new(1.0, 0.0, 0.0)),   // Right
            ([3, 2, 6, 7], Vec3::new(0.0, 1.0, 0.0)),   // Top
            ([4, 5, 1, 0], Vec3::new(0.0, -1.0, 0.0)),  // Bottom
        ];

        let mut curr = base_idx;
        for (f_idx, norm) in faces.iter() {
            let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            for i in 0..4 {
                let v = p[f_idx[i]];
                self.vertices.push(Vertex::new(
                    [v.x, v.y, v.z],
                    [norm.x, norm.y, norm.z],
                    [color.x, color.y, color.z, color.w],
                    uvs[i],
                ));
            }
            self.indices.extend_from_slice(&[
                curr, curr + 1, curr + 2,
                curr, curr + 2, curr + 3,
            ]);
            curr += 4;
        }
    }

    pub fn add_cylinder(
        &mut self,
        base: Vec3,
        axis: Vec3,
        radius: f32,
        length: f32,
        segments: usize,
        color: Vec4,
    ) {
        let norm_axis = axis.normalize();
        let ortho = if norm_axis.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0).cross(norm_axis).normalize()
        } else {
            Vec3::new(1.0, 0.0, 0.0).cross(norm_axis).normalize()
        };
        let ortho2 = norm_axis.cross(ortho).normalize();

        let top = base + norm_axis * length;
        let base_idx = self.vertices.len() as u32;

        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let (sin_a, cos_a) = angle.sin_cos();
            let normal = ortho * cos_a + ortho2 * sin_a;
            let p_bot = base + normal * radius;
            let p_top = top + normal * radius;

            let u = i as f32 / segments as f32;
            self.vertices.push(Vertex::new([p_bot.x, p_bot.y, p_bot.z], [normal.x, normal.y, normal.z], [color.x, color.y, color.z, color.w], [u, 0.0]));
            self.vertices.push(Vertex::new([p_top.x, p_top.y, p_top.z], [normal.x, normal.y, normal.z], [color.x, color.y, color.z, color.w], [u, 1.0]));
        }

        for i in 0..segments as u32 {
            let i0 = base_idx + i * 2;
            let i1 = i0 + 1;
            let i2 = i0 + 2;
            let i3 = i0 + 3;
            self.indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    pub fn export_to_obj_file(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        writeln!(file, "# Generated OBJ File for Apocalypse Vulkan FPS")?;
        
        for v in &self.vertices {
            writeln!(file, "v {:.5} {:.5} {:.5} {:.3} {:.3} {:.3}", 
                v.position[0], v.position[1], v.position[2],
                v.color[0], v.color[1], v.color[2]
            )?;
        }
        for v in &self.vertices {
            writeln!(file, "vn {:.4} {:.4} {:.4}", v.normal[0], v.normal[1], v.normal[2])?;
        }
        for v in &self.vertices {
            writeln!(file, "vt {:.4} {:.4}", v.uv[0], v.uv[1])?;
        }
        for chunk in self.indices.chunks(3) {
            if chunk.len() == 3 {
                let (i0, i1, i2) = (chunk[0] + 1, chunk[1] + 1, chunk[2] + 1);
                writeln!(file, "f {0}/{0}/{0} {1}/{1}/{1} {2}/{2}/{2}", i0, i1, i2)?;
            }
        }
        Ok(())
    }

    pub fn build(self) -> MeshData {
        MeshData {
            vertices: self.vertices,
            indices: self.indices,
        }
    }
}

pub fn create_akm_mesh() -> MeshData {
    let mut b = ModelBuilder::new();
    let dark_metal = Vec4::new(0.12, 0.12, 0.13, 1.0);
    let wood = Vec4::new(0.38, 0.20, 0.08, 1.0);
    let worn_steel = Vec4::new(0.25, 0.25, 0.27, 1.0);

    // Receiver (Corps central de l'AKM)
    b.add_box(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.025, 0.04, 0.20), dark_metal);
    // Dust cover (Haut du récepteur)
    b.add_box(Vec3::new(0.0, 0.045, -0.02), Vec3::new(0.022, 0.015, 0.16), worn_steel);
    // Wooden Handguard (Garde-main en bois)
    b.add_box(Vec3::new(0.0, 0.01, 0.28), Vec3::new(0.026, 0.035, 0.12), wood);
    // Gas tube (Tube d'emprunt de gaz supérieur)
    b.add_cylinder(Vec3::new(0.0, 0.04, 0.18), Vec3::new(0.0, 0.0, 1.0), 0.012, 0.22, 8, dark_metal);
    // Barrel (Canon de l'AKM 7.62mm)
    b.add_cylinder(Vec3::new(0.0, 0.015, 0.18), Vec3::new(0.0, 0.0, 1.0), 0.010, 0.42, 10, worn_steel);
    // Muzzle Brake (Frein de bouche biseauté AKM)
    b.add_box(Vec3::new(0.0, 0.015, 0.60), Vec3::new(0.012, 0.014, 0.025), dark_metal);
    // Front sight post (Guidon avant)
    b.add_box(Vec3::new(0.0, 0.05, 0.55), Vec3::new(0.005, 0.025, 0.01), dark_metal);
    // Rear sight leaf (Hausse arrière réglable)
    b.add_box(Vec3::new(0.0, 0.055, 0.15), Vec3::new(0.008, 0.01, 0.03), dark_metal);
    // Curved Banana Magazine (Chargeur courbé 30 coups 7.62x39mm)
    b.add_box(Vec3::new(0.0, -0.10, 0.08), Vec3::new(0.018, 0.08, 0.035), dark_metal);
    b.add_box(Vec3::new(0.0, -0.17, 0.11), Vec3::new(0.018, 0.05, 0.035), dark_metal);
    // Pistol grip (Poignée pistolet bakélite/bois)
    b.add_box(Vec3::new(0.0, -0.09, -0.12), Vec3::new(0.018, 0.07, 0.028), wood);
    // Wooden Stock (Crosse en bois classique)
    b.add_box(Vec3::new(0.0, -0.02, -0.32), Vec3::new(0.022, 0.045, 0.16), wood);
    b.add_box(Vec3::new(0.0, -0.05, -0.44), Vec3::new(0.020, 0.06, 0.04), dark_metal); // Plaque de couche
    // Charging handle & Bolt (Culasse et levier d'armement latéral)
    b.add_box(Vec3::new(0.03, 0.03, 0.02), Vec3::new(0.015, 0.008, 0.008), worn_steel);
    // Trigger & Guard
    b.add_box(Vec3::new(0.0, -0.05, -0.07), Vec3::new(0.005, 0.015, 0.01), dark_metal);

    b.build()
}

pub fn create_m4a1_mesh() -> MeshData {
    let mut b = ModelBuilder::new();
    let tactical_black = Vec4::new(0.09, 0.09, 0.10, 1.0);
    let charcoal = Vec4::new(0.18, 0.18, 0.20, 1.0);
    let dark_grey = Vec4::new(0.25, 0.25, 0.26, 1.0);

    // Upper & Lower Receiver
    b.add_box(Vec3::new(0.0, 0.01, 0.0), Vec3::new(0.022, 0.045, 0.18), tactical_black);
    // Picatinny Quad Rail Handguard (Garde-main RIS tactique)
    b.add_box(Vec3::new(0.0, 0.02, 0.26), Vec3::new(0.024, 0.024, 0.14), charcoal);
    // Handguard rails ribs
    b.add_box(Vec3::new(0.0, 0.045, 0.26), Vec3::new(0.012, 0.004, 0.14), dark_grey);
    b.add_box(Vec3::new(0.025, 0.02, 0.26), Vec3::new(0.004, 0.012, 0.14), dark_grey);
    b.add_box(Vec3::new(-0.025, 0.02, 0.26), Vec3::new(0.004, 0.012, 0.14), dark_grey);
    // Barrel (Canon 14.5 pouces 5.56 NATO)
    b.add_cylinder(Vec3::new(0.0, 0.02, 0.18), Vec3::new(0.0, 0.0, 1.0), 0.009, 0.38, 10, dark_grey);
    // Birdcage Flash Hider (Cache-flamme M4A1)
    b.add_cylinder(Vec3::new(0.0, 0.02, 0.55), Vec3::new(0.0, 0.0, 1.0), 0.012, 0.04, 8, tactical_black);
    // Triangle Front Sight Gas Block
    b.add_box(Vec3::new(0.0, 0.065, 0.40), Vec3::new(0.006, 0.035, 0.02), tactical_black);
    // Rear Carry Handle / Sight (ou optique)
    b.add_box(Vec3::new(0.0, 0.07, -0.05), Vec3::new(0.010, 0.02, 0.10), tactical_black);
    // STANAG Magazine (Chargeur droit 30 balles 5.56mm)
    b.add_box(Vec3::new(0.0, -0.12, 0.04), Vec3::new(0.016, 0.09, 0.035), charcoal);
    // Tactical Pistol Grip (Poignée texturée A2)
    b.add_box(Vec3::new(0.0, -0.08, -0.11), Vec3::new(0.016, 0.065, 0.025), tactical_black);
    // Buffer Tube & Retractable Crane Stock (Crosse télescopique)
    b.add_cylinder(Vec3::new(0.0, 0.02, -0.18), Vec3::new(0.0, 0.0, -1.0), 0.015, 0.18, 8, dark_grey);
    b.add_box(Vec3::new(0.0, 0.0, -0.32), Vec3::new(0.022, 0.055, 0.12), tactical_black);
    // Forward Assist & Ejection Port Cover
    b.add_box(Vec3::new(0.025, 0.03, -0.04), Vec3::new(0.008, 0.008, 0.015), dark_grey);

    b.build()
}

pub fn create_colt1911_mesh() -> MeshData {
    let mut b = ModelBuilder::new();
    let brushed_steel = Vec4::new(0.35, 0.36, 0.38, 1.0);
    let wood_grip = Vec4::new(0.35, 0.18, 0.08, 1.0);
    let dark_metal = Vec4::new(0.14, 0.14, 0.15, 1.0);

    // Frame (Carcasse du pistolet)
    b.add_box(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.018, 0.02, 0.12), brushed_steel);
    // Slide (Glissière / Culasse mobile .45 ACP)
    b.add_box(Vec3::new(0.0, 0.035, 0.02), Vec3::new(0.019, 0.022, 0.14), brushed_steel);
    // Barrel inside slide (Canon)
    b.add_cylinder(Vec3::new(0.0, 0.035, 0.14), Vec3::new(0.0, 0.0, 1.0), 0.008, 0.025, 8, dark_metal);
    // Rear & Front Sight (Mire et Guidon 3 points)
    b.add_box(Vec3::new(0.0, 0.06, 0.14), Vec3::new(0.004, 0.006, 0.006), dark_metal);
    b.add_box(Vec3::new(0.0, 0.06, -0.09), Vec3::new(0.008, 0.006, 0.006), dark_metal);
    // Grip handle (Poignée)
    b.add_box(Vec3::new(0.0, -0.07, -0.04), Vec3::new(0.017, 0.06, 0.025), dark_metal);
    // Classic Diamond Checkered Wood Grip Panels (Plaquettes en bois)
    b.add_box(Vec3::new(0.019, -0.07, -0.04), Vec3::new(0.002, 0.05, 0.020), wood_grip);
    b.add_box(Vec3::new(-0.019, -0.07, -0.04), Vec3::new(0.002, 0.05, 0.020), wood_grip);
    // Hammer (Chien arrière)
    b.add_box(Vec3::new(0.0, 0.04, -0.11), Vec3::new(0.006, 0.012, 0.01), dark_metal);
    // Beavertail Grip Safety (Pédale de sécurité arrière)
    b.add_box(Vec3::new(0.0, 0.01, -0.08), Vec3::new(0.012, 0.01, 0.02), brushed_steel);
    // Trigger & Trigger Guard
    b.add_box(Vec3::new(0.0, -0.02, 0.03), Vec3::new(0.004, 0.012, 0.008), dark_metal);
    // Magazine Base plate (Talon du chargeur 7 coups)
    b.add_box(Vec3::new(0.0, -0.132, -0.038), Vec3::new(0.018, 0.004, 0.026), dark_metal);

    b.build()
}

pub fn create_bot_mesh() -> MeshData {
    let mut b = ModelBuilder::new();
    let armor_dark = Vec4::new(0.15, 0.17, 0.16, 1.0);
    let camo_cloth = Vec4::new(0.28, 0.32, 0.22, 1.0);
    let visor_glow = Vec4::new(1.0, 0.15, 0.1, 1.0);
    let boots_skin = Vec4::new(0.1, 0.09, 0.08, 1.0);

    // Torso (Gilet pare-balles tactique)
    b.add_box(Vec3::new(0.0, 1.15, 0.0), Vec3::new(0.24, 0.28, 0.15), armor_dark);
    b.add_box(Vec3::new(0.0, 1.10, 0.12), Vec3::new(0.18, 0.14, 0.05), camo_cloth); // Poches de chargeurs
    // Head & Tactical Helmet (Casque tactique avec visière menaçante rouge)
    b.add_box(Vec3::new(0.0, 1.62, 0.0), Vec3::new(0.14, 0.15, 0.14), armor_dark);
    b.add_box(Vec3::new(0.0, 1.62, 0.13), Vec3::new(0.11, 0.04, 0.02), visor_glow); // Visière lumineuse
    // Left & Right Arms (Bras avec tenue militaire)
    b.add_box(Vec3::new(-0.34, 1.15, 0.05), Vec3::new(0.08, 0.26, 0.08), camo_cloth);
    b.add_box(Vec3::new(0.34, 1.15, 0.05), Vec3::new(0.08, 0.26, 0.08), camo_cloth);
    // Hands / Gloves
    b.add_box(Vec3::new(-0.34, 0.82, 0.12), Vec3::new(0.06, 0.07, 0.06), boots_skin);
    b.add_box(Vec3::new(0.34, 0.82, 0.12), Vec3::new(0.06, 0.07, 0.06), boots_skin);
    // Legs (Pantalon treillis)
    b.add_box(Vec3::new(-0.14, 0.50, 0.0), Vec3::new(0.09, 0.35, 0.10), camo_cloth);
    b.add_box(Vec3::new(0.14, 0.50, 0.0), Vec3::new(0.09, 0.35, 0.10), camo_cloth);
    // Boots (Rangers de combat)
    b.add_box(Vec3::new(-0.14, 0.08, 0.04), Vec3::new(0.10, 0.08, 0.14), boots_skin);
    b.add_box(Vec3::new(0.14, 0.08, 0.04), Vec3::new(0.10, 0.08, 0.14), boots_skin);

    b.build()
}

pub fn create_barrel_mesh() -> MeshData {
    let mut b = ModelBuilder::new();
    let red_barrel = Vec4::new(0.85, 0.12, 0.10, 1.0);
    let hazard_yellow = Vec4::new(0.9, 0.75, 0.1, 1.0);
    let dark_metal = Vec4::new(0.2, 0.2, 0.22, 1.0);

    // Corps du baril explosif
    b.add_cylinder(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.38, 0.95, 16, red_barrel);
    // Anneaux de renfort
    b.add_cylinder(Vec3::new(0.0, 0.28, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.40, 0.05, 16, dark_metal);
    b.add_cylinder(Vec3::new(0.0, 0.62, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.40, 0.05, 16, dark_metal);
    // Bande de danger explosif
    b.add_cylinder(Vec3::new(0.0, 0.42, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.385, 0.12, 16, hazard_yellow);

    b.build()
}

pub fn ensure_asset_obj_files(models_dir: &Path) {
    let _ = fs::create_dir_all(models_dir);

    let akm_path = models_dir.join("akm.obj");
    if !akm_path.exists() {
        let _ = create_akm_mesh().export_to_obj_file(&akm_path);
    }

    let m4a1_path = models_dir.join("m4a1.obj");
    if !m4a1_path.exists() {
        let _ = create_m4a1_mesh().export_to_obj_file(&m4a1_path);
    }

    let colt_path = models_dir.join("colt1911.obj");
    if !colt_path.exists() {
        let _ = create_colt1911_mesh().export_to_obj_file(&colt_path);
    }

    let bot_path = models_dir.join("enemy_bot.obj");
    if !bot_path.exists() {
        let _ = create_bot_mesh().export_to_obj_file(&bot_path);
    }

    let barrel_path = models_dir.join("barrel.obj");
    if !barrel_path.exists() {
        let _ = create_barrel_mesh().export_to_obj_file(&barrel_path);
    }
}

pub fn load_obj_or_fallback(path: &Path, fallback_fn: fn() -> MeshData) -> MeshData {
    if path.exists() {
        if let Ok((models, _materials)) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS) {
            if !models.is_empty() {
                let mut all_vertices = Vec::new();
                let mut all_indices = Vec::new();

                for m in models {
                    let mesh = &m.mesh;
                    let num_vertices = mesh.positions.len() / 3;
                    let base_idx = all_vertices.len() as u32;

                    for i in 0..num_vertices {
                        let pos = [
                            mesh.positions[i * 3],
                            mesh.positions[i * 3 + 1],
                            mesh.positions[i * 3 + 2],
                        ];
                        let norm = if !mesh.normals.is_empty() && mesh.normals.len() > i * 3 + 2 {
                            [mesh.normals[i * 3], mesh.normals[i * 3 + 1], mesh.normals[i * 3 + 2]]
                        } else {
                            [0.0, 1.0, 0.0]
                        };
                        let uv = if !mesh.texcoords.is_empty() && mesh.texcoords.len() > i * 2 + 1 {
                            [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
                        } else {
                            [0.0, 0.0]
                        };
                        let col = if !mesh.vertex_color.is_empty() && mesh.vertex_color.len() > i * 3 + 2 {
                            [
                                mesh.vertex_color[i * 3],
                                mesh.vertex_color[i * 3 + 1],
                                mesh.vertex_color[i * 3 + 2],
                                1.0,
                            ]
                        } else {
                            [0.7, 0.7, 0.7, 1.0]
                        };
                        all_vertices.push(Vertex::new(pos, norm, col, uv));
                    }

                    for idx in &mesh.indices {
                        all_indices.push(base_idx + *idx);
                    }
                }

                if !all_vertices.is_empty() && !all_indices.is_empty() {
                    return MeshData {
                        vertices: all_vertices,
                        indices: all_indices,
                    };
                }
            }
        }
    }
    fallback_fn()
}
