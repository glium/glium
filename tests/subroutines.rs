#[macro_use]
extern crate glium;

use glium::{Surface};
use glium::index::PrimitiveType;
use glium::program::{ ShaderStage, is_subroutine_supported };
use glium::backend::Facade;
use glium::DrawError;

mod support;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[test]
fn subroutine_bindings_simple() {
    let display = support::build_display();
    if !is_subroutine_supported(display.get_context()) {
        println!("Backend does not support subroutines");
        return
    };

    let program = program!(&display,
        150 => {
            vertex: "
                #version 150

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",

            fragment: "
                #version 150
                #extension GL_ARB_shader_subroutine : require

                out vec4 fragColor;
                subroutine vec4 color_t();

                subroutine uniform color_t Color;

                subroutine(color_t)
                vec4 ColorRed()
                {
                  return vec4(1, 0, 0, 1);
                }

                subroutine(color_t)
                vec4 ColorBlue()
                {
                  return vec4(0, 0, 1, 1);
                }

                void main()
                {
                    fragColor = Color();
                }
            "
        },
    ).unwrap();
    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);

    let uniforms = uniform!(
        Color: ("ColorBlue", ShaderStage::Fragment),
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (0, 0, 255, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(0, 0, 255, 255));

    display.assert_no_error(None);

    let uniforms = uniform!(
        Color: ("ColorRed", ShaderStage::Fragment),
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn subroutine_bindings_explicit_location() {
    let display = support::build_display();
    if !is_subroutine_supported(display.get_context()) {
        println!("Backend does not support subroutines");
        return
    };

    let program = program!(&display,
        150 => {
            vertex: "
                #version 150

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",

            fragment: "
                #version 150
                #extension GL_ARB_shader_subroutine : require
                #extension GL_ARB_explicit_uniform_location : require
                #extension GL_ARB_explicit_attrib_location : require

                out vec4 fragColor;
                subroutine vec4 color_t();

                layout(location = 5) subroutine uniform color_t Color;

                subroutine(color_t)
                vec4 ColorRed()
                {
                  return vec4(1, 0, 0, 1);
                }

                subroutine(color_t)
                vec4 ColorBlue()
                {
                  return vec4(0, 0, 1, 1);
                }

                void main()
                {
                    fragColor = Color();
                }
            "
        },
    ).unwrap();
    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);

    let uniforms = uniform!(
        Color: ("ColorBlue", ShaderStage::Fragment),
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (0, 0, 255, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(0, 0, 255, 255));

    display.assert_no_error(None);

    let uniforms = uniform!(
        Color: ("ColorRed", ShaderStage::Fragment),
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

// Start of more complex tests with multiple uniforms and such.

fn build_program_complex<T: glutin::surface::SurfaceTypeTrait + glutin::surface::ResizeableSurface + 'static>(display: &glium::Display<T>) -> glium::Program {
    let program = program!(display,
        150 => {
            vertex: "
                #version 150

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",

            fragment: "
                #version 150
                #extension GL_ARB_shader_subroutine : require

                out vec4 fragColor;
                subroutine vec4 color_t();
                subroutine vec4 modify_t(vec4 color);

                subroutine uniform color_t Color;
                subroutine uniform modify_t Modify;

                subroutine(color_t)
                vec4 ColorRed()
                {
                  return vec4(1, 0, 0, 1);
                }

                subroutine(color_t)
                vec4 ColorBlue()
                {
                  return vec4(0, 0, 1, 1);
                }

                subroutine(modify_t)
                vec4 SwapRB(vec4 color)
                {
                  return vec4(color.b, color.g, color.r, color.a);
                }

                subroutine(modify_t)
                vec4 DeleteR(vec4 color)
                {
                  return vec4(0, color.g, color.b, color.a);
                }

                void main()
                {
                    vec4 color = Color();
                    fragColor = Modify(color);
                }
            "
        },
    ).unwrap();
    program
}

#[test]
fn subroutine_bindings_multi() {
    let display = support::build_display();
    if !is_subroutine_supported(display.get_context()) {
        println!("Backend does not support subroutines");
        return
    };

    let program = build_program_complex(&display);
    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);

    let uniforms = uniform!(
        Color: ("ColorBlue", ShaderStage::Fragment),
        Modify: ("DeleteR", ShaderStage::Fragment),
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (0, 0, 255, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(0, 0, 255, 255));

    display.assert_no_error(None);

    let uniforms = uniform!(
        Color: ("ColorRed", ShaderStage::Fragment),
        Modify: ("SwapRB", ShaderStage::Fragment),
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (0, 0, 255, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(0, 0, 255, 255));

    display.assert_no_error(None);
}

#[test]
fn not_all_subroutine_uniforms_set() {
    let display = support::build_display();
    if !is_subroutine_supported(display.get_context()) {
        println!("Backend does not support subroutines");
        return
    };

    let program = build_program_complex(&display);
    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);

    let uniforms = uniform!(
        Color: ("ColorBlue", ShaderStage::Fragment),
        // Not setting Modify on purpose
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()) {
                                  Err(DrawError::SubroutineUniformMissing{ .. }) => (),
                                  _ => panic!("Drawing should have errored")
                              }

    display.assert_no_error(None);
}

#[test]
fn mismatched_subroutines() {
    let display = support::build_display();
    if !is_subroutine_supported(display.get_context()) {
        println!("Backend does not support subroutines");
        return
    };

    let program = build_program_complex(&display);
    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);

    let uniforms = uniform!(
        Color: ("ColorBlue", ShaderStage::Fragment),
        Modify: ("ColorBlue", ShaderStage::Fragment)
    );
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    match texture.as_surface().draw(&vb, &indices, &program, &uniforms,
                              &Default::default()) {
                                  Err(DrawError::SubroutineNotFound{ .. }) => (),
                                  _ => panic!("Drawing should have errored")
                              }

    display.assert_no_error(None);
}
