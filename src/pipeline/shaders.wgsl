struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    return out;
}

// 16-byte aligned Uniform Struct
struct ColorSettings {
    exposure: f32, contrast: f32, highlights: f32, shadows: f32,
    whites: f32, blacks: f32, temperature: f32, tint: f32,
    saturation: f32, vibrance: f32, sharpness: f32, brightness: f32,
    dither_enabled: f32, dither_type: f32, dither_scale: f32, dither_threshold: f32,
    dither_color: f32, posterize_levels: f32, bayer_size: f32, grad_enabled: f32,
    stipple_min_size: f32, stipple_max_size: f32, padding1: f32, padding2: f32,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var<uniform> settings: ColorSettings;
@group(0) @binding(3) var t_curves: texture_2d<f32>;
@group(0) @binding(4) var t_gradient: texture_2d<f32>;

fn get_luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn white_noise(p: vec2<f32>) -> f32 {
    return hash22(p).x;
}

fn interleaved_gradient_noise(p: vec2<f32>) -> f32 {
    let magic = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(p, magic.xy)));
}

// BAYER MATRICES
var<private> bayer2: array<f32, 4> = array<f32, 4>(0.0, 0.5, 0.75, 0.25);
var<private> bayer3: array<f32, 9> = array<f32, 9>(
    0.0, 0.777, 0.333,
    0.555, 0.111, 0.888,
    0.222, 0.666, 0.444
);
var<private> bayer4: array<f32, 16> = array<f32, 16>(
    0.0, 0.5, 0.125, 0.625,
    0.75, 0.25, 0.875, 0.375,
    0.1875, 0.6875, 0.0625, 0.5625,
    0.9375, 0.4375, 0.8125, 0.3125
);
var<private> bayer8: array<u32, 64> = array<u32, 64>(
    0u, 32u, 8u, 40u, 2u, 34u, 10u, 42u,
    48u, 16u, 56u, 24u, 50u, 18u, 58u, 26u,
    12u, 44u, 4u, 36u, 14u, 46u, 6u, 38u,
    60u, 28u, 52u, 20u, 62u, 30u, 54u, 22u,
    3u, 35u, 11u, 43u, 1u, 33u, 9u, 41u,
    51u, 19u, 59u, 27u, 49u, 17u, 57u, 25u,
    15u, 47u, 7u, 39u, 13u, 45u, 5u, 37u,
    63u, 31u, 55u, 23u, 61u, 29u, 53u, 21u
);

fn get_bayer_threshold(p: vec2<f32>, size: i32) -> f32 {
    let s = u32(size);
    let x = u32(p.x) % s;
    let y = u32(p.y) % s;
    let idx = y * s + x;
    
    if (size == 2) { return bayer2[idx]; }
    if (size == 3) { return bayer3[idx]; }
    if (size == 4) { return bayer4[idx]; }
    if (size == 8) { return f32(bayer8[idx]) / 64.0; }
    return 0.5;
}

fn apply_dither_step(val: f32, noise: f32, levels: f32) -> f32 {
    if (levels > 1.5) {
        let lv = levels - 1.0;
        let scaled = val * lv;
        let floor_v = floor(scaled);
        let diff = scaled - floor_v;
        if (diff > noise) { return (floor_v + 1.0) / lv; }
        else { return floor_v / lv; }
    } else {
        if (val > noise) { return 1.0; } else { return 0.0; }
    }
}

fn apply_adjustments(in_color: vec3<f32>, uv: vec2<f32>, tex_size: vec2<f32>) -> vec3<f32> {
    var color = in_color;
    
    // 1. Exposure & White Balance
    color *= pow(2.0, settings.exposure);
    color.r *= (1.0 + settings.temperature * 0.4);
    color.b *= (1.0 - settings.temperature * 0.4);
    color.g *= (1.0 - settings.tint * 0.25);
    color.r *= (1.0 + settings.tint * 0.1);
    color.b *= (1.0 + settings.tint * 0.1);

    // 2. Contrast & Brightness
    color += settings.brightness;
    color = (color - 0.5) * settings.contrast + 0.5;

    // 3. Highlights, Shadows, Whites, Blacks
    let lum_pre = get_luminance(color);
    color += color * smoothstep(0.4, 0.8, lum_pre) * settings.highlights * 0.5;
    color += color * (1.0 - smoothstep(0.2, 0.6, lum_pre)) * settings.shadows * 0.5;
    color += (1.0 - smoothstep(0.0, 0.3, lum_pre)) * settings.blacks * 0.3;
    color += smoothstep(0.7, 1.0, lum_pre) * settings.whites * 0.3;

    // 4. Saturation & Vibrance
    let l_pre_sat = get_luminance(color);
    let color_sat = max(color.r, max(color.g, color.b)) - min(color.r, min(color.g, color.b));
    color = mix(vec3<f32>(l_pre_sat), color, settings.saturation + settings.vibrance * (1.0 - color_sat));

    // 5. RGB Curves
    color.r = textureSampleLevel(t_curves, s_diffuse, vec2<f32>(clamp(color.r, 0.0, 1.0), 0.5), 0.0).r;
    color.g = textureSampleLevel(t_curves, s_diffuse, vec2<f32>(clamp(color.g, 0.0, 1.0), 0.5), 0.0).g;
    color.b = textureSampleLevel(t_curves, s_diffuse, vec2<f32>(clamp(color.b, 0.0, 1.0), 0.5), 0.0).b;
    
    return color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_diffuse));
    var uv = in.tex_coords;
    let scale = settings.dither_scale;
    
    if (settings.dither_enabled > 0.5 && scale > 1.0) {
        uv = (floor(uv * tex_size / scale) * scale + (scale * 0.5)) / tex_size;
    }

    var color = textureSample(t_diffuse, s_diffuse, uv).rgb;
    
    if (settings.sharpness > 0.0) {
        let dx = 1.0 / tex_size.x;
        let dy = 1.0 / tex_size.y;
        let laplacian = (textureSample(t_diffuse, s_diffuse, uv + vec2<f32>(-dx, 0.0)).rgb + textureSample(t_diffuse, s_diffuse, uv + vec2<f32>(dx, 0.0)).rgb + textureSample(t_diffuse, s_diffuse, uv + vec2<f32>(0.0, -dy)).rgb + textureSample(t_diffuse, s_diffuse, uv + vec2<f32>(0.0, dy)).rgb - 4.0 * color);
        color = color - settings.sharpness * laplacian;
    }

    color = apply_adjustments(color, uv, tex_size);

    var final_color = color;

    if (settings.dither_enabled < 0.5) {
        if (settings.posterize_levels > 1.5) {
            let lv = settings.posterize_levels - 1.0;
            final_color = floor(final_color * lv + 0.5) / lv;
        }
    } else {
        let d_scale = max(1.0, scale);
        let screen_pos = floor(in.tex_coords * tex_size / d_scale);
        let d_type = i32(settings.dither_type);
        
        var noise = settings.dither_threshold;
        if (d_type == 2) { 
            noise = white_noise(screen_pos); 
        } else if (d_type == 3) {
            noise = get_bayer_threshold(screen_pos, i32(settings.bayer_size));
        } else if (d_type == 4) {
            noise = interleaved_gradient_noise(screen_pos);
        } else if (d_type == 5) {
            let j = hash22(screen_pos);
            noise = (j.x + j.y + interleaved_gradient_noise(screen_pos)) / 3.0;
        } else if (d_type == 6) {
            let n1 = interleaved_gradient_noise(screen_pos);
            let n2 = interleaved_gradient_noise(screen_pos + vec2<f32>(5.0, 3.0));
            noise = fract(n1 * 0.75 + n2 * 0.25);
        } else if (d_type == 7) {
            let n = interleaved_gradient_noise(screen_pos);
            noise = step(0.5, n) * 0.5 + 0.25; 
        } else if (d_type == 8) {
            let dx = get_luminance(textureSample(t_diffuse, s_diffuse, uv + vec2<f32>(1.0/tex_size.x, 0.0)).rgb) - get_luminance(color);
            let dy = get_luminance(textureSample(t_diffuse, s_diffuse, uv + vec2<f32>(0.0, 1.0/tex_size.y)).rgb) - get_luminance(color);
            let edge = clamp(abs(dx) + abs(dy), 0.0, 1.0);
            noise = mix(interleaved_gradient_noise(screen_pos), settings.dither_threshold, edge * 0.8);
                } else if (d_type == 9) {
                    let p = screen_pos * 0.4;
                    let n = sin(p.x) * cos(p.y) + sin(p.y * 0.5) * cos(p.x * 0.5);
                    noise = fract(n * 2.0 + interleaved_gradient_noise(screen_pos) * 0.5);
                }
        
                if (settings.dither_color > 0.5) {            final_color.r = apply_dither_step(color.r, noise, settings.posterize_levels);
            final_color.g = apply_dither_step(color.g, noise, settings.posterize_levels);
            final_color.b = apply_dither_step(color.b, noise, settings.posterize_levels);
        } else {
            final_color = vec3<f32>(apply_dither_step(get_luminance(color), noise, settings.posterize_levels));
        }
    }

    if (settings.grad_enabled > 0.5) {
        let lum = clamp(get_luminance(final_color), 0.0, 1.0);
        final_color = textureSample(t_gradient, s_diffuse, vec2<f32>(lum, 0.5)).rgb;
    }

    return vec4<f32>(clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}