use glow::HasContext;
use glow_glyph::{GlyphBrushBuilder, Scale, Section};

fn main() -> Result<(), String> {
    env_logger::init();

    // Open window and create a surface
    let event_loop = glutin::event_loop::EventLoop::new();

    let window_builder =
        glutin::window::WindowBuilder::new().with_resizable(false);

    let context = glutin::ContextBuilder::new()
        .with_srgb(true)
        .build_windowed(window_builder, &event_loop)
        .expect("Open window");

    let context =
        unsafe { context.make_current().expect("Make OpenGL context current") };

    let mut size = context.window().inner_size();

    // Initialize OpenGL
    let gl = glow::Context::from_loader_function(|s| {
        context.get_proc_address(s) as *const _
    });

    // Prepare glyph_brush
    let inconsolata: &[u8] = include_bytes!("Inconsolata-Regular.ttf");
    let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(inconsolata)
        .expect("Load fonts")
        .build(&gl);

    // Render loop
    context.window().request_redraw();

    unsafe {
        gl.enable(glow::FRAMEBUFFER_SRGB);
        gl.clear_color(0.4, 0.4, 0.4, 1.0);
    }

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = glutin::event_loop::ControlFlow::Exit,
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::Resized(new_size),
                ..
            } => {
                context.resize(new_size);

                size = new_size;
            }
            glutin::event::Event::RedrawRequested { .. } => {
                unsafe { gl.clear(glow::COLOR_BUFFER_BIT) }

                glyph_brush.queue(Section {
                    text: "Hello wgpu_glyph!",
                    screen_position: (30.0, 30.0),
                    color: [0.0, 0.0, 0.0, 1.0],
                    scale: Scale { x: 40.0, y: 40.0 },
                    bounds: (size.width as f32, size.height as f32),
                    ..Section::default()
                });

                glyph_brush.queue(Section {
                    text: "Hello wgpu_glyph!",
                    screen_position: (30.0, 90.0),
                    color: [1.0, 1.0, 1.0, 1.0],
                    scale: Scale { x: 40.0, y: 40.0 },
                    bounds: (size.width as f32, size.height as f32),
                    ..Section::default()
                });

                // Draw the text!
                glyph_brush
                    .draw_queued(&gl, size.width, size.height)
                    .expect("Draw queued");

                context.swap_buffers().expect("Swap buffers");
            }
            _ => {
                *control_flow = glutin::event_loop::ControlFlow::Wait;
            }
        }
    })
}
