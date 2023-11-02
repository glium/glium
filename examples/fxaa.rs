#[macro_use]
extern crate glium;

use std::{cell::RefCell};

use glium::{Display, Surface, framebuffer::SimpleFrameBuffer};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};
use winit::keyboard::{PhysicalKey, KeyCode};

mod support;

#[derive(Copy, Clone)]
struct SpriteVertex {
    position: [f32; 2],
    i_tex_coords: [f32; 2],
}
implement_vertex!(SpriteVertex, position, i_tex_coords);

pub struct Application {
    vertex_buffer: glium::vertex::VertexBufferAny,
    program: glium::Program,
    camera: support::camera::CameraState,

    fxaa_enabled: bool,
    fxaa_vertex_buffer: glium::VertexBuffer<SpriteVertex>,
    fxaa_index_buffer: glium::IndexBuffer<u16>,
    fxaa_program: glium::Program,

    target_color: RefCell<Option<glium::texture::Texture2d>>,
    target_depth: RefCell<Option<glium::framebuffer::DepthRenderBuffer>>,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium FXAA example";

    fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            // building the vertex and index buffers
            vertex_buffer: support::load_wavefront(&display, include_bytes!("support/teapot.obj")),
            // the program
            program: program!(display,
                140 => {
                    vertex: "
                        #version 140

                        uniform mat4 persp_matrix;
                        uniform mat4 view_matrix;

                        in vec3 position;
                        in vec3 normal;
                        out vec3 v_position;
                        out vec3 v_normal;

                        void main() {
                            v_position = position;
                            v_normal = normal;
                            gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                        }
                    ",

                    fragment: "
                        #version 140

                        in vec3 v_normal;
                        out vec4 f_color;

                        const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

                        void main() {
                            float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                            vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                            f_color = vec4(color, 1.0);
                        }
                    ",
                },

                110 => {
                    vertex: "
                        #version 110

                        uniform mat4 persp_matrix;
                        uniform mat4 view_matrix;

                        attribute vec3 position;
                        attribute vec3 normal;
                        varying vec3 v_position;
                        varying vec3 v_normal;

                        void main() {
                            v_position = position;
                            v_normal = normal;
                            gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                        }
                    ",

                    fragment: "
                        #version 110

                        varying vec3 v_normal;

                        const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

                        void main() {
                            float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                            vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                            gl_FragColor = vec4(color, 1.0);
                        }
                    ",
                },

                100 => {
                    vertex: "
                        #version 100

                        uniform lowp mat4 persp_matrix;
                        uniform lowp mat4 view_matrix;

                        attribute lowp vec3 position;
                        attribute lowp vec3 normal;
                        varying lowp vec3 v_position;
                        varying lowp vec3 v_normal;

                        void main() {
                            v_position = position;
                            v_normal = normal;
                            gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                        }
                    ",

                    fragment: "
                        #version 100

                        varying lowp vec3 v_normal;

                        const lowp vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

                        void main() {
                            lowp float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                            lowp vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                            gl_FragColor = vec4(color, 1.0);
                        }
                    ",
                },
            ).unwrap(),

            camera: support::camera::CameraState::new(),
            fxaa_enabled: true,

            fxaa_vertex_buffer: glium::VertexBuffer::new(display,
                &[
                    SpriteVertex { position: [-1.0, -1.0], i_tex_coords: [0.0, 0.0] },
                    SpriteVertex { position: [-1.0,  1.0], i_tex_coords: [0.0, 1.0] },
                    SpriteVertex { position: [ 1.0,  1.0], i_tex_coords: [1.0, 1.0] },
                    SpriteVertex { position: [ 1.0, -1.0], i_tex_coords: [1.0, 0.0] }
                ]
            ).unwrap(),

            fxaa_index_buffer: glium::index::IndexBuffer::new(display,
                glium::index::PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap(),

            fxaa_program: program!(display,
                100 => {
                    vertex: r"
                        #version 100

                        attribute vec2 position;
                        attribute vec2 i_tex_coords;

                        varying vec2 v_tex_coords;

                        void main() {
                            gl_Position = vec4(position, 0.0, 1.0);
                            v_tex_coords = i_tex_coords;
                        }
                    ",
                    fragment: r"
                        #version 100

                        precision mediump float;

                        uniform vec2 resolution;
                        uniform sampler2D tex;
                        uniform int enabled;

                        varying vec2 v_tex_coords;

                        #define FXAA_REDUCE_MIN   (1.0/ 128.0)
                        #define FXAA_REDUCE_MUL   (1.0 / 8.0)
                        #define FXAA_SPAN_MAX     8.0

                        vec4 fxaa(sampler2D tex, vec2 fragCoord, vec2 resolution,
                                    vec2 v_rgbNW, vec2 v_rgbNE,
                                    vec2 v_rgbSW, vec2 v_rgbSE,
                                    vec2 v_rgbM) {
                            vec4 color;
                            mediump vec2 inverseVP = vec2(1.0 / resolution.x, 1.0 / resolution.y);
                            vec3 rgbNW = texture2D(tex, v_rgbNW).xyz;
                            vec3 rgbNE = texture2D(tex, v_rgbNE).xyz;
                            vec3 rgbSW = texture2D(tex, v_rgbSW).xyz;
                            vec3 rgbSE = texture2D(tex, v_rgbSE).xyz;
                            vec4 texColor = texture2D(tex, v_rgbM);
                            vec3 rgbM  = texColor.xyz;
                            vec3 luma = vec3(0.299, 0.587, 0.114);
                            float lumaNW = dot(rgbNW, luma);
                            float lumaNE = dot(rgbNE, luma);
                            float lumaSW = dot(rgbSW, luma);
                            float lumaSE = dot(rgbSE, luma);
                            float lumaM  = dot(rgbM,  luma);
                            float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
                            float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

                            mediump vec2 dir;
                            dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
                            dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));

                            float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) *
                                                    (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);

                            float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
                            dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
                                        max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
                                        dir * rcpDirMin)) * inverseVP;

                            vec3 rgbA = 0.5 * (
                                texture2D(tex, fragCoord * inverseVP + dir * (1.0 / 3.0 - 0.5)).xyz +
                                texture2D(tex, fragCoord * inverseVP + dir * (2.0 / 3.0 - 0.5)).xyz);
                            vec3 rgbB = rgbA * 0.5 + 0.25 * (
                                texture2D(tex, fragCoord * inverseVP + dir * -0.5).xyz +
                                texture2D(tex, fragCoord * inverseVP + dir * 0.5).xyz);

                            float lumaB = dot(rgbB, luma);
                            if ((lumaB < lumaMin) || (lumaB > lumaMax))
                                color = vec4(rgbA, texColor.a);
                            else
                                color = vec4(rgbB, texColor.a);
                            return color;
                        }

                        void main() {
                            vec2 fragCoord = v_tex_coords * resolution;
                            vec4 color;
                            if (enabled != 0) {
                                vec2 inverseVP = 1.0 / resolution.xy;
                                mediump vec2 v_rgbNW = (fragCoord + vec2(-1.0, -1.0)) * inverseVP;
                                mediump vec2 v_rgbNE = (fragCoord + vec2(1.0, -1.0)) * inverseVP;
                                mediump vec2 v_rgbSW = (fragCoord + vec2(-1.0, 1.0)) * inverseVP;
                                mediump vec2 v_rgbSE = (fragCoord + vec2(1.0, 1.0)) * inverseVP;
                                mediump vec2 v_rgbM = vec2(fragCoord * inverseVP);
                                color = fxaa(tex, fragCoord, resolution, v_rgbNW, v_rgbNE, v_rgbSW,
                                                v_rgbSE, v_rgbM);
                            } else {
                                color = texture2D(tex, v_tex_coords);
                            }
                            gl_FragColor = color;
                        }
                    "
                }
            ).unwrap(),
            target_color: RefCell::new(None),
            target_depth: RefCell::new(None),
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        self.camera.update();

        // Creating the frame that's drawn on the window
        let mut target = display.draw();
        if self.fxaa_enabled {
            self.draw_fxaa(&mut target, display);
        } else {
            self.draw_model(&mut target);
        }
        target.finish().unwrap();
    }

    fn handle_window_event(&mut self, event: &winit::event::WindowEvent, _window: &winit::window::Window) {
        self.camera.process_input(&event);

        match event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => match event.state {
                winit::event::ElementState::Pressed => match event.physical_key {
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.fxaa_enabled = !self.fxaa_enabled;
                        println!("FXAA is now {}", if self.fxaa_enabled { "enabled" } else { "disabled" });
                    },
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    }
}

impl Application {
    fn draw_model<T: glium::Surface>(&self, target: &mut T) {
        let uniforms = uniform! {
            persp_matrix: self.camera.get_perspective(),
            view_matrix: self.camera.get_view(),
        };

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&self.vertex_buffer,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &self.program, &uniforms, &params).unwrap();
    }

    fn draw_fxaa(&mut self, target: &mut glium::Frame, display: &Display<WindowSurface>) {
        let target_dimensions = target.get_dimensions();
        let color_dimensions = {
            self.target_color.borrow().as_ref().map_or((0,0), |tex| (tex.get_width(), tex.get_height().unwrap()))
        };
        let depth_dimensions = {
            self.target_depth.borrow().as_ref().map_or((0,0), |tex| tex.get_dimensions())
        };
        let mut target_color = self.target_color.borrow_mut();
        let mut target_depth = self.target_depth.borrow_mut();

        if target_color.is_none() || color_dimensions != target_dimensions {
            let texture = glium::texture::Texture2d::empty(display,
                                                           target_dimensions.0,
                                                           target_dimensions.1).unwrap();
            *target_color = Some(texture);
        }
        let target_color = target_color.as_ref().unwrap();

        if target_depth.is_none() || depth_dimensions != target_dimensions  {
            let texture = glium::framebuffer::DepthRenderBuffer::new(display,
                                                                      glium::texture::DepthFormat::I24,
                                                                      target_dimensions.0,
                                                                      target_dimensions.1).unwrap();
            *target_depth = Some(texture);
        }
        let target_depth = target_depth.as_ref().unwrap();

        let mut framebuffer = SimpleFrameBuffer::with_depth_buffer(display, target_color, target_depth).unwrap();
        self.draw_model(&mut framebuffer);

        let uniforms = uniform! {
            tex: &*target_color,
            enabled: if self.fxaa_enabled { 1i32 } else { 0i32 },
            resolution: (target_dimensions.0 as f32, target_dimensions.1 as f32)
        };
        target.draw(&self.fxaa_vertex_buffer, &self.fxaa_index_buffer, &self.fxaa_program, &uniforms,
                    &Default::default()).unwrap();
    }
}


fn main() {
    println!("This example demonstrates FXAA. Is is an anti-aliasing technique done at the \
              post-processing stage. This example draws the teapot to a framebuffer and then \
              copies from the texture to the main framebuffer by applying a filter to it.\n\
              You can use the space bar to switch fxaa on and off.");
    State::<Application>::run_loop();
}
