pub struct Point2 {
    x: f32,
    y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Point2 {
        Point2 { x, y }
    }
}

pub struct Color(f32, f32, f32, f32);

impl Color {
    pub fn from_srgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color(r, g, b, a)
    }
}

pub struct Vertex {
    coords: Point2,
    color: Color,
}

//SHOULD BE IN CLOCKWISE ORDER
pub const VERTICES: &[f32; 8] = &[
    // Triangle 1
    -0.8, -0.8, //
    0.8, -0.8, //
    0.8, 0.8, //
    -0.8, 0.8, //
];

pub const INDICES: &[u16; 6] = &[0, 1, 2, 2, 3, 0];

pub const GRID_SIZE: f32 = 32.;
pub const UNIFORM_ARRAY: &[f32; 2] = &[GRID_SIZE, GRID_SIZE];
