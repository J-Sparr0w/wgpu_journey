use render_state::RenderState;
use wgpu_learn_02::{primitives::Point2, render_state, App};
use winit::window::WindowAttributes;

fn main() {
    // pollster::block_on(run());
    let mut renderer = RenderState::new();
    renderer.draw_square(Point2::new(-0.8, 0.8), 0.4);
    renderer.draw_square(Point2::new(0.8, 0.8), 0.2);
    renderer.draw_triangle_ccw(
        Point2::new(0.0, 0.0),
        Point2::new(0.4, 0.0),
        Point2::new(0.4, 0.4),
    );
    renderer.draw_line(Point2::new(0., -0.1), Point2::new(1., -1.));

    let mut app = App::init(
        Some(
            WindowAttributes::default()
                .with_title("WGPU LEARN")
                .with_theme(Some(winit::window::Theme::Dark)),
        ),
        Some(renderer),
    );
}
