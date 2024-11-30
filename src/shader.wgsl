

struct VertexInput {
    @location(0) pos: vec2f,
    @builtin(instance_index) instance: u32,
};

@group(0) @binding(0) var<uniform> grid: vec2f; 
@group(0) @binding(1) var<storage> cell_state: array<u32>;

struct VertexOutput {
    @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let i = f32(input.instance);
    let pos = input.pos;
    let cell = vec2f(i % grid.x, floor(i / grid.y)) ;
    let cell_offset = cell / grid * 2;

    let state = f32(cell_state[input.instance]);
    // setting position of all vertices of an instance to 0 = scaling the instance to 0
    let grid_pos = ((pos + 1) / grid) -1 + cell_offset;

    var output: VertexOutput;
    output.pos = vec4f(grid_pos * state, 0, 1);
    return  output;
}



@fragment
fn fs_main() -> @location(0) vec4f {
    let color = vec4f(0.8, 0.8, 0.5, 1) ;
    return color;
}


