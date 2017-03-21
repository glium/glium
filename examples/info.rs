extern crate glium;

fn main() {
    use glium::{Api, Profile, Version};
    use glium::glutin;

    // building the display, ie. the main object
    let events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_visibility(false).build(&events_loop).unwrap();
    let display = glium::build(window).unwrap();

    let version = *display.get_opengl_version();
    let api = match version {
        Version(Api::Gl, _, _) => "OpenGL",
        Version(Api::GlEs, _, _) => "OpenGL ES"
    };

    println!("{} context verson: {}", api, display.get_opengl_version_string());

    print!("{} context flags:", api);
    if display.is_forward_compatible() {
        print!(" forward-compatible");
    }
    if display.is_debug() {
        print!(" debug");
    }
    if display.is_robust() {
        print!(" robustness");
    }
    print!("\n");

    if version >= Version(Api::Gl, 3, 2) {
        println!("{} profile mask: {}", api,
                 match display.get_opengl_profile() {
                     Some(Profile::Core) => "core",
                     Some(Profile::Compatibility) => "compatibility",
                     None => "unknown"
                 });
    }

    println!("{} robustness strategy: {}", api,
             if display.is_context_loss_possible() {
                 "lose"
             } else {
                 "none"
             });
    
    println!("{} context renderer: {}", api, display.get_opengl_renderer_string());
    println!("{} context vendor: {}", api, display.get_opengl_vendor_string());
}
