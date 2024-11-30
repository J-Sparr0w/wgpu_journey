use wgpu_learn_02::App;
use winit::window::WindowAttributes;

fn main() {
    // pollster::block_on(run());
    let mut app = App::init(Some(
        WindowAttributes::default()
            .with_title("WGPU LEARN")
            .with_theme(Some(winit::window::Theme::Dark)),
    ));
}
