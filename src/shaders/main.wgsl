struct GlobalUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    pad0: f32,
    sun_dir: vec3<f32>,
    pad1: f32,
    sun_color: vec3<f32>,
    ambient_intensity: f32,
    fog_color: vec3<f32>,
    fog_density: f32,
    muzzle_pos: vec3<f32>,
    muzzle_intensity: f32,
    muzzle_color: vec3<f32>,
    hit_flash: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: GlobalUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_p = uniforms.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_p.xyz;
    out.clip_position = uniforms.view_proj * world_p;
    
    // Transform normal with 3x3 model matrix
    let normal_matrix = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * in.normal);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let L = normalize(-uniforms.sun_dir);
    let V = normalize(uniforms.camera_pos - in.world_pos);
    let H = normalize(L + V);

    // Diffuse Sunlight
    let diff = max(dot(N, L), 0.0);
    let diffuse = uniforms.sun_color * diff;

    // Ambient light
    let ambient = vec3<f32>(0.28, 0.24, 0.22) * uniforms.ambient_intensity;

    // Specular highlight
    let spec = pow(max(dot(N, H), 0.0), 32.0);
    let specular = uniforms.sun_color * spec * 0.35;

    // Muzzle flash point light
    var muzzle_contrib = vec3<f32>(0.0);
    if (uniforms.muzzle_intensity > 0.01) {
        let m_diff = uniforms.muzzle_pos - in.world_pos;
        let m_dist = length(m_diff);
        if (m_dist < 18.0) {
            let m_dir = normalize(m_diff);
            let m_att = 1.0 / (1.0 + 0.3 * m_dist + 0.15 * m_dist * m_dist);
            let m_diffuse = max(dot(N, m_dir), 0.0);
            muzzle_contrib = uniforms.muzzle_color * m_diffuse * uniforms.muzzle_intensity * m_att;
        }
    }

    // Base color combined
    var lit_color = in.color.rgb * (diffuse + ambient + muzzle_contrib) + specular;

    // Hit flash reaction
    if (uniforms.hit_flash > 0.01) {
        lit_color = mix(lit_color, vec3<f32>(1.0, 0.2, 0.2), uniforms.hit_flash * 0.7);
    }

    // Distance Apocalyptic Fog
    let dist = length(uniforms.camera_pos - in.world_pos);
    let fog_factor = clamp(1.0 - exp(-dist * uniforms.fog_density), 0.0, 1.0);
    let final_color = mix(lit_color, uniforms.fog_color, fog_factor);

    return vec4<f32>(final_color, in.color.a);
}
