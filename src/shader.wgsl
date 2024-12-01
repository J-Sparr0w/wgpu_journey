

struct VertexInput {
    @location(0) pos: vec2f,
    @builtin(instance_index) instance: u32,
};


struct VertexOutput {
    @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let i = f32(input.instance);
    let pos = input.pos;


    var output: VertexOutput;
    output.pos = vec4f(pos, 0, 1);
    return  output;
}



@fragment
fn fs_main() -> @location(0) vec4f {
    let color = vec4f(0.8, 0.8, 0.5, 1) ;
    return color;
}


