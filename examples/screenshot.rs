#![feature(phase)]


#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

#[cfg(feature = "image")]
extern crate image;

use glium::Surface;

#[cfg(not(feature = "image"))]
fn main() {
    println!("You need to compile glium with the `image` feature in order to run this example");
}

#[cfg(feature = "image")]
fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_visibility(false)
        .build_glium()
        .unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[vertex_format]
        #[deriving(Copy)]
        struct Vertex {
            position: [f32, ..2],
            color: [f32, ..3],
        }

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]
        )
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TrianglesList(vec![0u16, 1, 2]));

    // compiling shaders and linking them together
    let program = glium::Program::new(&display,
        // vertex shader
        "
            #version 110

            uniform mat4 matrix;

            attribute vec2 position;
            attribute vec3 color;

            varying vec3 vColor;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0) * matrix;
                vColor = color;
            }
        ",

        // fragment shader
        "
            #version 110
            varying vec3 vColor;

            void main() {
                gl_FragColor = vec4(vColor, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // creating the uniforms structure
    #[uniforms]
    #[deriving(Copy)]
    struct Uniforms {
        matrix: [[f32, ..4], ..4],
    }
    
    // drawing once

    // building the uniforms
    let uniforms = Uniforms {
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
    target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &std::default::Default::default());
    target.finish();

    // reading the front buffer into an image
    let image: image::DynamicImage = display.read_front_buffer();
    let output = std::io::fs::File::create(&Path::new("glium-example-screenshot.png"));
    image.save(output, image::ImageFormat::PNG).unwrap();
}
