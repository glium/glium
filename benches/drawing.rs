#![cfg(feature = "unstable")]
#![feature(test)]

extern crate gl;
#[macro_use]
extern crate glium;
extern crate test;

use glium::DisplayBuild;
use glium::Surface;
use glium::glutin;

use test::Bencher;

use std::ffi::CStr;
use std::mem;
use std::ptr;

#[bench]
fn glium_triangle(b: &mut Bencher) {
    let display = glutin::WindowBuilder::new().build_glium().unwrap();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.5, -0.5], color: [0.0, 0.0, 1.0] },
            ]
        ).unwrap()
    };

    let program = program!(&display,
        100 => {
            vertex: "
                #version 100
                precision mediump float;

                attribute vec2 position;
                attribute vec3 color;

                varying vec3 v_color;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_color = color;
                }
            ",

            fragment: "
                #version 100
                precision mediump float;

                varying vec3 v_color;

                void main() {
                    gl_FragColor = vec4(v_color, 1.0);
                }
            ",
        },
    ).unwrap();

    b.iter(|| {
        for _ in (0 .. 10) {
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            target.draw(&vertex_buffer, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                        &program, &uniform!{}, &Default::default()).unwrap();
            target.finish().unwrap();
        }
    });
}

#[bench]
fn opengl_triangle(b: &mut Bencher) {
    static VERTEX_DATA: [f32; 15] = [
        -0.5, -0.5, 1.0, 0.0, 0.0,
        0.0, 0.5, 0.0, 1.0, 0.0,
        0.5, -0.5, 0.0, 0.0, 1.0
    ];

    const VS_SRC: &'static [u8] = b"
    #version 100
    precision mediump float;

    attribute vec2 position;
    attribute vec3 color;

    varying vec3 v_color;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        v_color = color;
    }
    \0";

    const FS_SRC: &'static [u8] = b"
    #version 100
    precision mediump float;

    varying vec3 v_color;

    void main() {
        gl_FragColor = vec4(v_color, 1.0);
    }
    \0";

    unsafe {
        let mut window = glutin::WindowBuilder::new().build().unwrap();
        unsafe { window.make_current().unwrap() };
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const i8].as_ptr(), ptr::null());
        gl::CompileShader(vs);

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const i8].as_ptr(), ptr::null());
        gl::CompileShader(fs);

        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        gl::UseProgram(program);

        let mut vb = mem::uninitialized();
        gl::GenBuffers(1, &mut vb);
        gl::BindBuffer(gl::ARRAY_BUFFER, vb);
        gl::BufferData(gl::ARRAY_BUFFER,
                           (VERTEX_DATA.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                           VERTEX_DATA.as_ptr() as *const _, gl::STATIC_DRAW);

        if gl::BindVertexArray::is_loaded() {
            let mut vao = mem::uninitialized();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        let pos_attrib = gl::GetAttribLocation(program, b"position\0".as_ptr() as *const _);
        let color_attrib = gl::GetAttribLocation(program, b"color\0".as_ptr() as *const _);
        gl::VertexAttribPointer(pos_attrib as gl::types::GLuint, 2, gl::FLOAT, 0,
                                    5 * mem::size_of::<f32>() as gl::types::GLsizei,
                                    ptr::null());
        gl::VertexAttribPointer(color_attrib as gl::types::GLuint, 3, gl::FLOAT, 0,
                                    5 * mem::size_of::<f32>() as gl::types::GLsizei,
                                    (2 * mem::size_of::<f32>()) as *const () as *const _);
        gl::EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
        gl::EnableVertexAttribArray(color_attrib as gl::types::GLuint);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        b.iter(|| {
            for _ in (0 .. 10) {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
                window.swap_buffers();
            }
        });
    }
}
