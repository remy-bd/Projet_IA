use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // in radians
    pub pitch: f32, // in radians
    pub fov: f32,   // in degrees (e.g. 75.0 base, 50.0 in ADS)
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    pub recoil_pitch: f32,
    pub recoil_yaw: f32,
    pub shake: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Self {
            position,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            fov: 75.0,
            aspect,
            znear: 0.05,
            zfar: 400.0,
            recoil_pitch: 0.0,
            recoil_yaw: 0.0,
            shake: Vec3::ZERO,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let total_pitch = (self.pitch + self.recoil_pitch).clamp(-1.54, 1.54);
        let total_yaw = self.yaw + self.recoil_yaw;
        Vec3::new(
            total_yaw.cos() * total_pitch.cos(),
            total_pitch.sin(),
            total_yaw.sin() * total_pitch.cos(),
        ).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        let eye = self.position + self.shake;
        let target = eye + self.forward();
        Mat4::look_at_rh(eye, target, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.znear, self.zfar)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn update_decay(&mut self, dt: f32) {
        self.recoil_pitch = (self.recoil_pitch - dt * 4.5).max(0.0);
        self.recoil_yaw *= (-dt * 8.0).exp();
        self.shake *= (-dt * 12.0).exp();
    }

    pub fn add_recoil(&mut self, pitch_up: f32, yaw_side: f32) {
        self.recoil_pitch += pitch_up;
        self.recoil_yaw += yaw_side;
    }

    pub fn add_shake(&mut self, amount: f32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let ox = rng.gen_range(-amount..amount);
        let oy = rng.gen_range(-amount..amount);
        let oz = rng.gen_range(-amount..amount);
        self.shake += Vec3::new(ox, oy, oz);
    }
}
