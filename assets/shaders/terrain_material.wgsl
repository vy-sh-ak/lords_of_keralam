struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
};

struct TerrainMaterial {
    params: vec4<f32>,
    layer0: vec4<f32>,
    layer1: vec4<f32>,
    layer2: vec4<f32>,
    layer3: vec4<f32>,
    tint0: vec4<f32>,
    tint1: vec4<f32>,
    tint2: vec4<f32>,
    tint3: vec4<f32>,
};

@group(3) @binding(0) var<uniform> material: TerrainMaterial;
@group(3) @binding(1) var texture0: texture_2d<f32>;
@group(3) @binding(2) var sampler0: sampler;
@group(3) @binding(3) var texture1: texture_2d<f32>;
@group(3) @binding(4) var sampler1: sampler;
@group(3) @binding(5) var texture2: texture_2d<f32>;
@group(3) @binding(6) var sampler2: sampler;
@group(3) @binding(7) var texture3: texture_2d<f32>;
@group(3) @binding(8) var sampler3: sampler;

fn layer_weight(start: f32, blend: f32, height: f32) -> f32 {
    if blend <= 0.0 {
        return select(0.0, 1.0, abs(height - start) < 0.001);
    }
    return 1.0 - smoothstep(0.0, blend, abs(height - start));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = material.params.x;
    let count = u32(material.params.w);

    let p = fract(in.world_position.xz * scale + 0.5);
    let height = in.world_position.y;

    var color = vec3(0.0);
    var total_weight = 0.0;

    // Layer 0
    if count > 0u {
        let w = layer_weight(material.layer0.x, material.layer0.y, height);
        let tint_str = material.layer0.z;
        let tex = textureSample(texture0, sampler0, p).rgb;
        let tinted = mix(tex, material.tint0.rgb, tint_str);
        color += tinted * w;
        total_weight += w;
    }

    // Layer 1
    if count > 1u {
        let w = layer_weight(material.layer1.x, material.layer1.y, height);
        let tint_str = material.layer1.z;
        let tex = textureSample(texture1, sampler1, p).rgb;
        let tinted = mix(tex, material.tint1.rgb, tint_str);
        color += tinted * w;
        total_weight += w;
    }

    // Layer 2
    if count > 2u {
        let w = layer_weight(material.layer2.x, material.layer2.y, height);
        let tint_str = material.layer2.z;
        let tex = textureSample(texture2, sampler2, p).rgb;
        let tinted = mix(tex, material.tint2.rgb, tint_str);
        color += tinted * w;
        total_weight += w;
    }

    // Layer 3
    if count > 3u {
        let w = layer_weight(material.layer3.x, material.layer3.y, height);
        let tint_str = material.layer3.z;
        let tex = textureSample(texture3, sampler3, p).rgb;
        let tinted = mix(tex, material.tint3.rgb, tint_str);
        color += tinted * w;
        total_weight += w;
    }

    if total_weight > 0.0 {
        color = color / total_weight;
    }

    // fallback: gray when no layers apply
    let alpha = min(total_weight, 1.0);
    return vec4(color, alpha);
}
