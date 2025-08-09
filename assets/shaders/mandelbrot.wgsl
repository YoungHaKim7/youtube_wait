// Mandelbrot fragment shader
// Inputs: time, zoom, offset, resolution

struct MandelParams {
    time: f32,
    zoom: f32,
    offset: vec2<f32>,
    aspect: f32,
}

@group(1) @binding(0)
var<uniform> params: MandelParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Map uv (0..1) to complex plane with aspect correction
    var uv = (in.uv - vec2<f32>(0.5, 0.5));
    uv.x *= params.aspect;

    let scale = pow(0.98, params.time) * params.zoom; // continuous zoom
    let c = vec2<f32>(uv.x * scale + params.offset.x, uv.y * scale + params.offset.y);

    var z = vec2<f32>(0.0, 0.0);
    var i: i32 = 0;
    let max_iter: i32 = 256;

    // Iterate z = z^2 + c
    loop {
        if (i >= max_iter) { break; }
        let x = (z.x * z.x - z.y * z.y) + c.x;
        let y = (2.0 * z.x * z.y) + c.y;
        z = vec2<f32>(x, y);
        if (dot(z, z) > 4.0) { break; }
        i = i + 1;
    }

    let t = f32(i) / f32(max_iter);
    // Smooth coloring
    let color = vec3<f32>(0.5 + 0.5*cos(6.2831*(vec3<f32>(0.0, 0.33, 0.67) + t)), t, t*t);
    return vec4<f32>(color, 1.0);
}
