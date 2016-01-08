#[macro_use]
extern crate glium;

fn main() {
    use glium::{Api, CapabilitiesSource, DisplayBuild, Profile, Version};
    use glium::backend::Facade;
    use glium::glutin;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_visibility(false)
        .build_glium()
        .unwrap();

    let version = *display.get_opengl_version();
    let api = match version {
        Version(Api::Gl, _, _) => "OpenGL",
        Version(Api::GlEs, _, _) => "OpenGL ES"
    };

    let caps = display.get_context().get_capabilities();

    println!("{} context verson: {}", api, caps.version);

    print!("{} context flags:", api);
    if caps.forward_compatible {
        print!(" forward-compatible");
    }
    if caps.debug {
        print!(" debug");
    }
    if caps.robustness {
        print!(" robustness");
    }
    print!("\n");

    if version >= Version(Api::Gl, 3, 2) {
        println!("{} profile mask: {}", api,
                 match caps.profile {
                     Some(Profile::Core) => "core",
                     Some(Profile::Compatibility) => "compatibility",
                     None => "unknown"
                 });
    }

    println!("{} robustness strategy: {}", api,
             if caps.can_lose_context {
                 "lose"
             } else {
                 "none"
             });
    
    println!("{} context renderer: {}", api, caps.renderer);
    println!("{} context vendor: {}", api, caps.vendor);
}
