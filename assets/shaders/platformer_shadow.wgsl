#import bevy_pbr::forward_io::VertexOutput

struct PlatformerShadowCasterInfo {
    radius: f32,
    translation: vec3<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> caster: PlatformerShadowCasterInfo;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var material_color_sampler: sampler;

fn distance_to_caster(position: vec4<f32>, caster: PlatformerShadowCasterInfo) -> f32 {
    return length(position.xz - caster.translation.xz);
}

fn one_if_beneath_caster(position: vec4<f32>, caster: PlatformerShadowCasterInfo) -> f32 {
    return max(0, sign(caster.translation.y - position.y));
}

fn shadow_multiplier_for_caster(position: vec4<f32>, caster: PlatformerShadowCasterInfo) -> f32 {
    let distance = distance_to_caster(position, caster);

    let beneath = one_if_beneath_caster(position, caster);

    let shadow_intensity = (beneath * sqrt(max((caster.radius * caster.radius) - (distance * distance), 0.0))) / caster.radius;

    return 1.0 - shadow_intensity;
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4f {
    let base_color = textureSample(material_color_texture, material_color_sampler, in.uv);

    let shadow_mult = shadow_multiplier_for_caster(in.world_position, caster);

    return vec4f((base_color.xyz * shadow_mult), base_color.a);
}
