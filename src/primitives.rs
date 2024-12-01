use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl<T: Into<f32>> Add<T> for Point2 {
    type Output = Point2;

    fn add(mut self, rhs: T) -> Self::Output {
        let rhs: f32 = rhs.into();
        self.x += rhs;
        self.y += rhs;

        self
    }
}

impl<T: Into<f32>> Sub<T> for Point2 {
    type Output = Point2;

    fn sub(mut self, rhs: T) -> Self::Output {
        let rhs: f32 = rhs.into();
        self.x -= rhs;
        self.y -= rhs;

        self
    }
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Point2 {
        Point2 { x, y }
    }
    pub fn add_to_x<T: Into<f32>>(mut self, rhs: T) -> Self {
        self.x += rhs.into();
        self
    }
    pub fn add_to_y<T: Into<f32>>(mut self, rhs: T) -> Self {
        self.y += rhs.into();
        self
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
