#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};
use glium::index::PrimitiveType;

fn main() {
    // building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_visible(true);
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex buffer with no vertices
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            dummy: f32,
        }

        implement_vertex!(Vertex, dummy);

        glium::VertexBuffer::<Vertex>::empty(&display, 0).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u8, 1, 2]).unwrap();

    // loading SPIR-V fragment shader with GLSL vertex shader and linking them together
    let program = glium::Program::new(&display,
        glium::program::ProgramCreationInput::SpirV {
            vertex_shader: include_bytes!("spirv/vert.spv"),
            fragment_shader: include_bytes!("spirv/frag.spv"),
            outputs_srgb: true,
            uses_point_size: false,
        }).unwrap();

    // drawing once

    // building the uniforms
    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ]
    };

    // drawing a frame
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
    target.finish().unwrap();

    // reading the front buffer into an image
    let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
    let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv();
    image.save("glium-example-screenshot.png").unwrap();
}
