#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct PlatformerShadowCasterInfo {
    radius: f32,
    translation_xz: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<storage, read> caster: PlatformerShadowCasterInfo;

fn distance_to_caster(position: vec4<f32>, caster: PlatformerShadowCasterInfo) -> f32 {
    return length(position.xz - caster.translation_xz);
}

fn shadow_multiplier_for_caster(position: vec4<f32>, caster: PlatformerShadowCasterInfo) -> f32 {
    let distance = distance_to_caster(position, caster);

    let shadow_intensity = sqrt(max((caster.radius * caster.radius) - (distance * distance), 0.0)) / caster.radius;

    return 1.0 - shadow_intensity;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;

    let shadow_mult = shadow_multiplier_for_caster(pbr_input.world_position, caster);

    out.color = pbr_input.material.base_color * shadow_mult;
#endif

    return out;
}
