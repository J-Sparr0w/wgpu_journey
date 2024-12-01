use crate::primitives::Point2;

#[derive(Debug, Clone, Default)]
pub struct RenderState {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub primitives_count: u32,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            primitives_count: 0,
        }
    }
    pub fn draw_square(&mut self, top_left: Point2, width: f32) {
        let top_right = top_left.add_to_x(width);
        let bottom_left = top_left.add_to_y(-width);
        let bottom_right = top_left.add_to_x(width).add_to_y(-width);
        let offset_start = self.vertices.len() / 2;

        self.vertices.push(top_left.x);
        self.vertices.push(top_left.y); //
        self.vertices.push(bottom_left.x);
        self.vertices.push(bottom_left.y); //
        self.vertices.push(bottom_right.x);
        self.vertices.push(bottom_right.y); //
        self.vertices.push(top_right.x);
        self.vertices.push(top_right.y); //

        // dbg!(&self.vertices[offset_start..]);
        dbg!(&self.vertices[..]);

        let index_offset_start = self.indices.len();

        self.indices.push((offset_start + 0) as u16);
        self.indices.push((offset_start + 1) as u16);
        self.indices.push((offset_start + 2) as u16);
        self.indices.push((offset_start + 0) as u16);
        self.indices.push((offset_start + 2) as u16);
        self.indices.push((offset_start + 3) as u16);

        // dbg!(&self.indices[index_offset_start..]);
        dbg!(&self.indices[..]);
        self.primitives_count += 2;
    }
    pub fn draw_triangle_ccw(&mut self, p0: Point2, p1: Point2, p2: Point2) {
        // p0,p1 and p2 must be counter clockwise to be visible on the screen
        let offset_start = self.vertices.len() / 2;
        self.vertices.push(p0.x);
        self.vertices.push(p0.y);
        self.vertices.push(p1.x);
        self.vertices.push(p1.y);
        self.vertices.push(p2.x);
        self.vertices.push(p2.y);

        // dbg!(&self.vertices[..]);

        self.indices.push((offset_start + 0) as u16);
        self.indices.push((offset_start + 1) as u16);
        self.indices.push((offset_start + 2) as u16);
        self.primitives_count += 1;
        // dbg!(&self.indices[..]);
    }
    pub fn draw_line(&mut self, p0: Point2, p1: Point2) {
        let stroke_width = 0.005;

        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;

        let distance = f32::sqrt(dx * dx + dy * dy);

        let ux = -dy / distance;
        let uy = dx / distance;

        let x = p0.x + stroke_width * ux;
        let y = p0.y + stroke_width * uy;
        let top_left = Point2::new(x, y);
        let x = p0.x - stroke_width * ux;
        let y = p0.y - stroke_width * uy;
        let bottom_left = Point2::new(x, y);

        let x = p1.x - stroke_width * ux;
        let y = p1.y - stroke_width * uy;
        let bottom_right = Point2::new(x, y);

        let x = p1.x + stroke_width * ux;
        let y = p1.y + stroke_width * uy;
        let top_right = Point2::new(x, y);

        let offset_start = self.vertices.len() / 2;

        self.vertices.push(top_left.x);
        self.vertices.push(top_left.y); //
        self.vertices.push(bottom_left.x);
        self.vertices.push(bottom_left.y); //
        self.vertices.push(bottom_right.x);
        self.vertices.push(bottom_right.y); //
        self.vertices.push(top_right.x);
        self.vertices.push(top_right.y); //

        // dbg!(&self.vertices[offset_start..]);
        dbg!(&self.vertices[..]);

        let index_offset_start = self.indices.len();

        self.indices.push((offset_start + 0) as u16);
        self.indices.push((offset_start + 1) as u16);
        self.indices.push((offset_start + 2) as u16);
        self.indices.push((offset_start + 0) as u16);
        self.indices.push((offset_start + 2) as u16);
        self.indices.push((offset_start + 3) as u16);

        // dbg!(&self.indices[index_offset_start..]);
        dbg!(&self.indices[..]);
        self.primitives_count += 2;
    }
}
