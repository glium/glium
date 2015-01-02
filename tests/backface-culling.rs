#![feature(phase)]
#![feature(unboxed_closures)]

#[phase(plugin)]
extern crate glium_macros;

extern crate glutin;
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn cull_clockwise() {
    let display = support::build_display();

    let vertex_buffer = {
        #[vertex_format]
        #[deriving(Copy)]
        struct Vertex {
            position: [f32, ..2],
        }

        glium::VertexBuffer::new(&display, vec![
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ])
    };

    // first triangle covers the top-left side of the screen and is clockwise
    // second triangle covers the bottom-right side of the screen and is ccw
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TrianglesList(vec![0u16, 1, 2, 1, 2, 3]));

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
        .unwrap();

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullClockWise,
            .. std::default::Default::default()
        });
    target.finish();

    let read_back: Vec<Vec<(f32, f32, f32, f32)>> = display.read_front_buffer();
    assert_eq!(read_back[0][0], (0.0, 0.0, 0.0, 0.0));
    assert_eq!(read_back.last().unwrap().last().unwrap(), &(1.0, 0.0, 0.0, 1.0));
    
    display.assert_no_error();
}

#[test]
fn cull_counterclockwise() {
    let display = support::build_display();

    let vertex_buffer = {
        #[vertex_format]
        #[deriving(Copy)]
        struct Vertex {
            position: [f32, ..2],
        }

        glium::VertexBuffer::new(&display, vec![
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ])
    };

    // first triangle covers the top-left side of the screen and is clockwise
    // second triangle covers the bottom-right side of the screen and is ccw
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TrianglesList(vec![0u16, 1, 2, 1, 2, 3]));

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
        .unwrap();

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullCounterClockWise,
            .. std::default::Default::default()
        });
    target.finish();

    let read_back: Vec<Vec<(f32, f32, f32, f32)>> = display.read_front_buffer();
    assert_eq!(read_back[0][0], (1.0, 0.0, 0.0, 1.0));
    assert_eq!(read_back.last().unwrap().last().unwrap(), &(0.0, 0.0, 0.0, 0.0));
    
    display.assert_no_error();
}

#[test]
fn cull_clockwise_trianglestrip() {
    let display = support::build_display();

    let vertex_buffer = {
        #[vertex_format]
        #[deriving(Copy)]
        struct Vertex {
            position: [f32, ..2],
        }

        glium::VertexBuffer::new(&display, vec![
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ])
    };

    // both triangles are clockwise
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TriangleStrip(vec![0u16, 1, 2, 3]));

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
        .unwrap();

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullClockWise,
            .. std::default::Default::default()
        });
    target.finish();

    let read_back: Vec<Vec<(f32, f32, f32, f32)>> = display.read_front_buffer();
    assert_eq!(read_back[0][0], (0.0, 0.0, 0.0, 0.0));
    assert_eq!(read_back.last().unwrap().last().unwrap(), &(0.0, 0.0, 0.0, 0.0));
    
    display.assert_no_error();
}

#[test]
fn cull_counterclockwise_trianglestrip() {
    let display = support::build_display();

    let vertex_buffer = {
        #[vertex_format]
        #[deriving(Copy)]
        struct Vertex {
            position: [f32, ..2],
        }

        glium::VertexBuffer::new(&display, vec![
            Vertex { position: [-1.0,  1.0] },      // top-left
            Vertex { position: [ 1.0,  1.0] },      // top-right
            Vertex { position: [-1.0, -1.0] },      // bottom-left
            Vertex { position: [ 1.0, -1.0] }       // bottom-right
        ])
    };

    // both triangles are clockwise
    let index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TriangleStrip(vec![0u16, 1, 2, 3]));

    let program = glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 110

            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
        .unwrap();

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniforms::EmptyUniforms,
        &glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullCounterClockWise,
            .. std::default::Default::default()
        });
    target.finish();

    let read_back: Vec<Vec<(f32, f32, f32, f32)>> = display.read_front_buffer();
    assert_eq!(read_back[0][0], (1.0, 0.0, 0.0, 1.0));
    assert_eq!(read_back.last().unwrap().last().unwrap(), &(1.0, 0.0, 0.0, 1.0));
    
    display.assert_no_error();
}
