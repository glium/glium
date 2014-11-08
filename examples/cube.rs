//! Gfx-rs's triangle example rewritten for Glium.

#![feature(phase)]

#[phase(plugin)]
extern crate glium_macros;

extern crate cgmath;
extern crate glutin;
extern crate glium;

use glium::{DisplayBuild, Surface};

use cgmath::FixedArray;
use cgmath::{Matrix, Point3, Vector3};
use cgmath::{Transform, AffineMatrix3};

fn main() {
    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new().with_vsync().build_glium().unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[vertex_format]
        struct Vertex {
            pos: [f32, ..3],
            tex_coord: [f32, ..2],
        }

        glium::VertexBuffer::new(&display, 
            vec![
                // top (0.0, 0.0, 1.0)
                Vertex { pos: [-1.0, -1.0,  1.0], tex_coord: [0.0, 0.0] },
                Vertex { pos: [ 1.0, -1.0,  1.0], tex_coord: [1.0, 0.0] },
                Vertex { pos: [ 1.0,  1.0,  1.0], tex_coord: [1.0, 1.0] },
                Vertex { pos: [-1.0,  1.0,  1.0], tex_coord: [0.0, 1.0] },
                // bottom (0.0, 0.0, -1.0)
                Vertex { pos: [ 1.0,  1.0, -1.0], tex_coord: [0.0, 0.0] },
                Vertex { pos: [-1.0,  1.0, -1.0], tex_coord: [1.0, 0.0] },
                Vertex { pos: [-1.0, -1.0, -1.0], tex_coord: [1.0, 1.0] },
                Vertex { pos: [ 1.0, -1.0, -1.0], tex_coord: [0.0, 1.0] },
                // right (1.0, 0.0, 0.0)
                Vertex { pos: [ 1.0, -1.0, -1.0], tex_coord: [0.0, 0.0] },
                Vertex { pos: [ 1.0,  1.0, -1.0], tex_coord: [1.0, 0.0] },
                Vertex { pos: [ 1.0,  1.0,  1.0], tex_coord: [1.0, 1.0] },
                Vertex { pos: [ 1.0, -1.0,  1.0], tex_coord: [0.0, 1.0] },
                // left (-1.0, 0.0, 0.0)
                Vertex { pos: [-1.0,  1.0,  1.0], tex_coord: [0.0, 0.0] },
                Vertex { pos: [-1.0, -1.0,  1.0], tex_coord: [1.0, 0.0] },
                Vertex { pos: [-1.0, -1.0, -1.0], tex_coord: [1.0, 1.0] },
                Vertex { pos: [-1.0,  1.0, -1.0], tex_coord: [0.0, 1.0] },
                // front (0.0, 1.0, 0.0)
                Vertex { pos: [-1.0,  1.0, -1.0], tex_coord: [0.0, 0.0] },
                Vertex { pos: [ 1.0,  1.0, -1.0], tex_coord: [1.0, 0.0] },
                Vertex { pos: [ 1.0,  1.0,  1.0], tex_coord: [1.0, 1.0] },
                Vertex { pos: [-1.0,  1.0,  1.0], tex_coord: [0.0, 1.0] },
                // back (0.0, -1.0, 0.0)
                Vertex { pos: [ 1.0, -1.0,  1.0], tex_coord: [0.0, 0.0] },
                Vertex { pos: [-1.0, -1.0,  1.0], tex_coord: [1.0, 0.0] },
                Vertex { pos: [-1.0, -1.0, -1.0], tex_coord: [1.0, 1.0] },
                Vertex { pos: [ 1.0, -1.0, -1.0], tex_coord: [0.0, 1.0] },
            ]
        )
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, glium::index_buffer::TrianglesList(vec![
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 16, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20u8, // back
    ]));

    // compiling shaders and linking them together
    let program = glium::Program::new(&display,
        // vertex shader
        "
            #version 120
            attribute vec3 pos;
            attribute vec2 tex_coord;
            varying vec2 v_TexCoord;
            uniform mat4 transform;
            void main() {
                v_TexCoord = tex_coord;
                gl_Position = transform * vec4(pos, 1.0);
            }
        ",

        // fragment shader
        "
            #version 120
            varying vec2 v_TexCoord;
            uniform sampler2D color;
            void main() {
                vec4 tex = texture2D(color, v_TexCoord);
                float blend = dot(v_TexCoord-vec2(0.5,0.5), v_TexCoord-vec2(0.5,0.5));
                gl_FragColor = mix(tex, vec4(0.0,0.0,0.0,0.0), blend*1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // building the texture
    let texture = glium::Texture2d::new(&display, vec![vec![(0x20u8, 0xA0u8, 0xC0u8, 0x00u8)]]);

    // creating the uniforms structure
    #[uniforms]
    struct Uniforms<'a> {
        transform: [[f32, ..4], ..4],
        color: glium::uniforms::Sampler<'a, glium::Texture2d>,
    }

    // creating the matrix
    let view: AffineMatrix3<f32> = Transform::look_at(
        &Point3::new(1.5f32, -5.0, 3.0),
        &Point3::new(0f32, 0.0, 0.0),
        &Vector3::unit_z(),
    );
    let aspect = {
        let (w, h) = display.get_framebuffer_dimensions();
        w as f32 / h as f32
    };
    let proj = cgmath::perspective(cgmath::deg(45.0f32), aspect, 1.0, 10.0);

    // the main loop
    // each cycle will draw once
    'main: loop {
        // building the uniforms
        let uniforms = Uniforms {
            transform: proj.mul_m(&view.mat).into_fixed(),
            color: glium::uniforms::Sampler(&texture, glium::uniforms::SamplerBehavior {
                wrap_function: (glium::uniforms::Clamp, glium::uniforms::Clamp,
                                glium::uniforms::Clamp),
                minify_filter: glium::uniforms::Linear,
                .. std::default::Default::default()
            }),
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);
        target.clear_depth(1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &glium::DrawParameters {
            depth_function: Some(glium::IfLessOrEqual),
            .. std::default::Default::default()
        });
        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events().into_iter() {
            match event {
                glutin::Closed => break 'main,
                _ => ()
            }
        }
    }
}
