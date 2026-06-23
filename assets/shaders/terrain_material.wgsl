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

fn sample_layer(tex: texture_2d<f32>, samp: sampler, layer: vec4<f32>, tint: vec4<f32>, world_pos: vec4<f32>, height: f32) -> vec3<f32> {
    let scale = layer.w;
    let p = fract(world_pos.xz * scale + 0.5);
    let w = layer_weight(layer.x, layer.y, height);
    let tint_str = layer.z;
    let tex_color = textureSample(tex, samp, p).rgb;
    return mix(tex_color, tint.rgb, tint_str) * w;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let count = u32(material.params.w);
    let height = in.world_position.y;

    var color = vec3(0.0);
    var total_weight = 0.0;

    // Layer 0
    if count > 0u {
        let c = sample_layer(texture0, sampler0, material.layer0, material.tint0, in.world_position, height);
        color += c;
        total_weight += layer_weight(material.layer0.x, material.layer0.y, height);
    }

    // Layer 1
    if count > 1u {
        let c = sample_layer(texture1, sampler1, material.layer1, material.tint1, in.world_position, height);
        color += c;
        total_weight += layer_weight(material.layer1.x, material.layer1.y, height);
    }

    // Layer 2
    if count > 2u {
        let c = sample_layer(texture2, sampler2, material.layer2, material.tint2, in.world_position, height);
        color += c;
        total_weight += layer_weight(material.layer2.x, material.layer2.y, height);
    }

    // Layer 3
    if count > 3u {
        let c = sample_layer(texture3, sampler3, material.layer3, material.tint3, in.world_position, height);
        color += c;
        total_weight += layer_weight(material.layer3.x, material.layer3.y, height);
    }

    if total_weight > 0.0 {
        color = color / total_weight;
    }

    // fallback: gray when no layers apply
    let alpha = min(total_weight, 1.0);
    return vec4(color, alpha);
}
