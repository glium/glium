#[macro_use]
extern crate glium;

use glium::{index, Surface};
use glium::index::PrimitiveType;

mod support;

fn build_program<T: glutin::surface::SurfaceTypeTrait + glutin::surface::ResizeableSurface + 'static>(display: &glium::Display<T>) -> glium::Program {
    program!(display,
        110 => {
            vertex: "
                #version 110

                attribute vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
        }
    ).unwrap()
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[test]
fn triangles_list() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn triangle_strip() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                          &[0u16, 1, 2, 3]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn triangle_fan() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [0.0,  0.0] },
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleFan,
                                          &[0u16, 1, 2, 4, 3, 1]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &indices, &program, &glium::uniforms::EmptyUniforms,
                              &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn get_primitives_type() {
    let display = support::build_display();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                          &[0u16, 1, 2, 3]).unwrap();

    assert_eq!(indices.get_primitives_type(), glium::index::PrimitiveType::TriangleStrip);

    display.assert_no_error(None);
}

#[test]
fn get_indices_type_u8() {
    let display = support::build_display();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                          &[0u8, 1, 2, 3]).unwrap();

    assert_eq!(indices.get_indices_type(), glium::index::IndexType::U8);

    display.assert_no_error(None);
}

#[test]
fn get_indices_type_u16() {
    let display = support::build_display();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                          &[0u16, 1, 2, 3]).unwrap();

    assert_eq!(indices.get_indices_type(), glium::index::IndexType::U16);

    display.assert_no_error(None);
}

#[test]
fn get_indices_type_u32() {
    let display = support::build_display();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                          &[0u32, 1, 2, 3]);

    let indices = match indices {
        Err(glium::index::BufferCreationError::IndexTypeNotSupported) => return,
        Ok(i) => i,
        e => e.unwrap()
    };

    assert_eq!(indices.get_indices_type(), glium::index::IndexType::U32);

    display.assert_no_error(None);
}

#[test]
fn triangles_list_noindices() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &index::NoIndices(index::PrimitiveType::TrianglesList),
                                                     &program, &glium::uniforms::EmptyUniforms,
                                                     &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn triangle_strip_noindices() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &index::NoIndices(index::PrimitiveType::TriangleStrip),
                                                     &program, &glium::uniforms::EmptyUniforms,
                                                     &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn triangle_fan_noindices() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [ 0.0,  0.0] },
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [ 1.0, -1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [-1.0,  1.0] },
    ]).unwrap();

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, &index::NoIndices(index::PrimitiveType::TriangleFan),
                                                     &program, &glium::uniforms::EmptyUniforms,
                                                     &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn empty_index_buffer() {
    let display = support::build_display();

    let _indices = glium::IndexBuffer::new(&display, PrimitiveType::TriangleFan,
                                           &Vec::<u16>::new()).unwrap();

    display.assert_no_error(None);
}

#[test]
fn indexbuffer_slice_out_of_range() {
    let display = support::build_display();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 2, 1, 3]).unwrap();

    assert!(indices.slice(5 .. 8).is_none());
    assert!(indices.slice(2 .. 11).is_none());
    assert!(indices.slice(12 .. 13).is_none());

    display.assert_no_error(None);
}

#[test]
fn indexbuffer_slice_draw() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 3, 2, 0, 1, 3]).unwrap();

    let texture1 = support::build_renderable_texture(&display);
    texture1.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture1.as_surface().draw(&vb, &indices.slice(3 .. 6).unwrap(), &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture1.read();
    assert_eq!(data[0][0], (0, 0, 0, 0));
    assert_eq!(data.last().unwrap().last().unwrap(), &(255, 0, 0, 255));


    let texture2 = support::build_renderable_texture(&display);
    texture2.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture2.as_surface().draw(&vb, &indices.slice(0 .. 3).unwrap(), &program,
                &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture2.read();
    assert_eq!(data[0][0], (255, 0, 0, 255));
    assert_eq!(data.last().unwrap().last().unwrap(), &(0, 0, 0, 0));


    display.assert_no_error(None);
}

#[test]
fn multidraw_array() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let multidraw = glium::index::DrawCommandsNoIndicesBuffer::empty(&display, 1);
    let multidraw = match multidraw {
        Ok(buf) => buf,
        Err(_) => return
    };

    multidraw.write(&[
        glium::index::DrawCommandNoIndices {
            count: 4,
            instance_count: 1,
            first_index: 0,
            base_instance: 0,
        }
    ]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, multidraw.with_primitive_type(PrimitiveType::TriangleStrip),
                              &program, &uniform!{}, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn multidraw_elements() {
    let display = support::build_display();
    let program = build_program(&display);

    let vb = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0] }, Vertex { position: [1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] }, Vertex { position: [1.0, -1.0] },
    ]).unwrap();

    let indices = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                          &[0u16, 1, 2, 1, 3, 2]).unwrap();

    let multidraw = glium::index::DrawCommandsIndicesBuffer::empty(&display, 1);
    let multidraw = match multidraw {
        Ok(buf) => buf,
        Err(_) => return
    };

    multidraw.write(&[
        glium::index::DrawCommandIndices {
            count: 6,
            instance_count: 1,
            first_index: 0,
            base_vertex: 0,
            base_instance: 0,
        }
    ]);

    let texture = support::build_renderable_texture(&display);
    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    texture.as_surface().draw(&vb, multidraw.with_index_buffer(&indices),
                              &program, &uniform!{}, &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}
