struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
};

struct TerrainMaterial {
    params: vec4<f32>,
    tint_color: vec4<f32>,
};

@group(3) @binding(0) var<uniform> material: TerrainMaterial;
@group(3) @binding(1) var water_texture: texture_2d<f32>;
@group(3) @binding(2) var water_sampler: sampler;

fn height_fade(start: f32, end: f32, height: f32) -> f32 {
    let range = end - start;
    if range <= 0.0 {
        return select(0.0, 1.0, height <= start);
    }
    return 1.0 - smoothstep(start, end, height);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = material.params.x;
    let height_start = material.params.y;
    let height_end = material.params.z;
    let tint_strength = material.params.w;

    let p = fract(in.world_position.xz * scale + 0.5);
    let height = in.world_position.y;

    var color = textureSample(water_texture, water_sampler, p).rgb;

    let fade = height_fade(height_start, height_end, height);
    color = mix(color, material.tint_color.rgb, tint_strength);
    color = color * fade;

    return vec4(color, fade);
}
