#[macro_use]
extern crate glium;

use cgmath::SquareMatrix;
use winit::keyboard::{PhysicalKey, KeyCode};
use std::time::Instant;
use glium::{Surface, Display, VertexBuffer, IndexBuffer, Program, texture::DepthTexture2d};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

mod support;

fn create_box(display: &glium::Display<WindowSurface>) -> (glium::VertexBuffer<Vertex>, glium::IndexBuffer<u16>) {
    let box_vertex_buffer = glium::VertexBuffer::new(display, &[
        // Max X
        Vertex { position: [ 0.5,-0.5,-0.5, 1.0], normal: [ 1.0, 0.0, 0.0, 0.0] },
        Vertex { position: [ 0.5,-0.5, 0.5, 1.0], normal: [ 1.0, 0.0, 0.0, 0.0] },
        Vertex { position: [ 0.5, 0.5, 0.5, 1.0], normal: [ 1.0, 0.0, 0.0, 0.0] },
        Vertex { position: [ 0.5, 0.5,-0.5, 1.0], normal: [ 1.0, 0.0, 0.0, 0.0] },
        // Min X
        Vertex { position: [-0.5,-0.5,-0.5, 1.0], normal: [-1.0, 0.0, 0.0, 0.0] },
        Vertex { position: [-0.5, 0.5,-0.5, 1.0], normal: [-1.0, 0.0, 0.0, 0.0] },
        Vertex { position: [-0.5, 0.5, 0.5, 1.0], normal: [-1.0, 0.0, 0.0, 0.0] },
        Vertex { position: [-0.5,-0.5, 0.5, 1.0], normal: [-1.0, 0.0, 0.0, 0.0] },
        // Max Y
        Vertex { position: [-0.5, 0.5,-0.5, 1.0], normal: [ 0.0, 1.0, 0.0, 0.0] },
        Vertex { position: [ 0.5, 0.5,-0.5, 1.0], normal: [ 0.0, 1.0, 0.0, 0.0] },
        Vertex { position: [ 0.5, 0.5, 0.5, 1.0], normal: [ 0.0, 1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, 0.5, 0.5, 1.0], normal: [ 0.0, 1.0, 0.0, 0.0] },
        // Min Y
        Vertex { position: [-0.5,-0.5,-0.5, 1.0], normal: [ 0.0,-1.0, 0.0, 0.0] },
        Vertex { position: [-0.5,-0.5, 0.5, 1.0], normal: [ 0.0,-1.0, 0.0, 0.0] },
        Vertex { position: [ 0.5,-0.5, 0.5, 1.0], normal: [ 0.0,-1.0, 0.0, 0.0] },
        Vertex { position: [ 0.5,-0.5,-0.5, 1.0], normal: [ 0.0,-1.0, 0.0, 0.0] },
        // Max Z
        Vertex { position: [-0.5,-0.5, 0.5, 1.0], normal: [ 0.0, 0.0, 1.0, 0.0] },
        Vertex { position: [-0.5, 0.5, 0.5, 1.0], normal: [ 0.0, 0.0, 1.0, 0.0] },
        Vertex { position: [ 0.5, 0.5, 0.5, 1.0], normal: [ 0.0, 0.0, 1.0, 0.0] },
        Vertex { position: [ 0.5,-0.5, 0.5, 1.0], normal: [ 0.0, 0.0, 1.0, 0.0] },
        // Min Z
        Vertex { position: [-0.5,-0.5,-0.5, 1.0], normal: [ 0.0, 0.0,-1.0, 0.0] },
        Vertex { position: [ 0.5,-0.5,-0.5, 1.0], normal: [ 0.0, 0.0,-1.0, 0.0] },
        Vertex { position: [ 0.5, 0.5,-0.5, 1.0], normal: [ 0.0, 0.0,-1.0, 0.0] },
        Vertex { position: [-0.5, 0.5,-0.5, 1.0], normal: [ 0.0, 0.0,-1.0, 0.0] },
        ]).unwrap();

    let mut indexes = Vec::new();
    for face in 0..6u16 {
        indexes.push(4 * face + 0);
        indexes.push(4 * face + 1);
        indexes.push(4 * face + 2);
        indexes.push(4 * face + 0);
        indexes.push(4 * face + 2);
        indexes.push(4 * face + 3);
    }
    let box_index_buffer = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indexes).unwrap();
    (box_vertex_buffer, box_index_buffer)
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
}
implement_vertex!(Vertex, position, normal);

#[derive(Clone, Debug)]
struct ModelData {
    model_matrix: cgmath::Matrix4<f32>,
    depth_mvp: cgmath::Matrix4<f32>,
    color: [f32; 4],
}
impl ModelData {
    pub fn color(c: [f32; 3]) -> Self {
        Self {
            model_matrix: cgmath::Matrix4::identity(),
            depth_mvp: cgmath::Matrix4::identity(),
            color: [c[0], c[1], c[2], 1.0],
        }
    }
    pub fn scale(mut self, s: f32) -> Self {
        self.model_matrix = self.model_matrix * cgmath::Matrix4::from_scale(s);
        self
    }
    pub fn translate(mut self, t: [f32; 3]) -> Self {
        self.model_matrix = self.model_matrix * cgmath::Matrix4::from_translation(t.into());
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct DebugVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(DebugVertex, position, tex_coords);
impl DebugVertex {
    pub fn new(position: [f32; 2], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            tex_coords,
        }
    }
}

struct Application {
    pub start: Instant,
    pub camera_t: f64,
    pub camera_rotating: bool,
    pub light_t: f64,
    pub light_rotating: bool,
    pub shadow_texture: DepthTexture2d,

    pub model_vertex_buffer: VertexBuffer<Vertex>,
    pub model_index_buffer: IndexBuffer<u16>,

    pub shadow_map_shaders: Program,
    pub render_shaders: Program,
    pub debug_vertex_buffer: VertexBuffer<DebugVertex>,
    pub debug_index_buffer: IndexBuffer<u16>,
    pub debug_shadow_map_shaders: Program,

    pub model_data: [ModelData; 4],
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium shadow_mapping example";

    fn new(display: &Display<WindowSurface>) -> Self {
        let shadow_map_size = 1024;

        // Create the boxes to render in the scene
        let (model_vertex_buffer, model_index_buffer) = create_box(display);
        let model_data = [
            ModelData::color([0.4, 0.4, 0.4]).translate([0.0, -2.5, 0.0]).scale(5.0),
            ModelData::color([0.6, 0.1, 0.1]).translate([0.0, 0.252, 0.0]).scale(0.5),
            ModelData::color([0.1, 0.6, 0.1]).translate([0.9, 0.5, 0.1]).scale(0.5),
            ModelData::color([0.1, 0.1, 0.6]).translate([-0.8, 0.75, 0.1]).scale(0.5),
        ];

        let shadow_map_shaders = glium::Program::from_source(
            display,
            // Vertex Shader
            "
                #version 330 core
                in vec4 position;
                uniform mat4 depth_mvp;
                void main() {
                gl_Position = depth_mvp * position;
                }
            ",
            // Fragement Shader
            "
                #version 330 core
                layout(location = 0) out float fragmentdepth;
                void main(){
                    fragmentdepth = gl_FragCoord.z;
                }
            ",
            None).unwrap();

        let render_shaders = glium::Program::from_source(
            display,
            // Vertex Shader
            "
                #version 330 core

                uniform mat4 mvp;
                uniform mat4 depth_bias_mvp;
                uniform mat4 model_matrix;
                uniform vec4 model_color;

                in vec4 position;
                in vec4 normal;

                out vec4 shadow_coord;
                out vec4 model_normal;

                void main() {
                    gl_Position =  mvp * position;
                    model_normal = model_matrix * normal;
                    shadow_coord = depth_bias_mvp * position;
                }
            ",
            // Fragement Shader
            "
                #version 330 core

                uniform sampler2DShadow shadow_map;
                uniform vec3 light_loc;
                uniform vec4 model_color;

                in vec4 shadow_coord;
                in vec4 model_normal;

                out vec4 color;

                void main() {
                    vec3 light_color = vec3(1,1,1);
                    float bias = 0.0; // Geometry does not require bias

                    float lum = max(dot(normalize(model_normal.xyz), normalize(light_loc)), 0.0);

                    float visibility = texture(shadow_map, vec3(shadow_coord.xy, (shadow_coord.z-bias)/shadow_coord.w));

                    color = vec4(max(lum * visibility, 0.05) * model_color.rgb * light_color, 1.0);
                }
            ",
            None).unwrap();

        // Debug Resources (for displaying shadow map)
        let debug_vertex_buffer = glium::VertexBuffer::new(
            display,
            &[
                DebugVertex::new([0.25, -1.0], [0.0, 0.0]),
                DebugVertex::new([0.25, -0.25], [0.0, 1.0]),
                DebugVertex::new([1.0, -0.25], [1.0, 1.0]),
                DebugVertex::new([1.0, -1.0], [1.0, 0.0]),
            ],
        ).unwrap();
        let debug_index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 0, 2, 3],
        ).unwrap();
        let debug_shadow_map_shaders = glium::Program::from_source(
            display,
            // Vertex Shader
            "
                #version 140
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",
            // Fragement Shader
            "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    f_color = vec4(texture(tex, v_tex_coords).rgb, 1.0);
                }
            ",
            None).unwrap();

        let shadow_texture = glium::texture::DepthTexture2d::empty(display, shadow_map_size, shadow_map_size).unwrap();

        let start = Instant::now();

        let light_t: f64 = 8.7;
        let light_rotating = false;
        let camera_t: f64 = 8.22;
        let camera_rotating = false;

        Self {
            start,
            camera_t,
            camera_rotating,
            light_t,
            light_rotating,
            shadow_texture,
            model_data,
            model_vertex_buffer,
            model_index_buffer,
            shadow_map_shaders,
            render_shaders,
            debug_vertex_buffer,
            debug_index_buffer,
            debug_shadow_map_shaders,
        }
    }

    fn handle_window_event(&mut self, event: &winit::event::WindowEvent, _window: &winit::window::Window) {
        match event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => if event.state == winit::event::ElementState::Pressed {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::KeyC => self.camera_rotating = !self.camera_rotating,
                        KeyCode::KeyL => self.light_rotating = !self.light_rotating,
                        _ => {}
                    }
                }
            },
            _ => return,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        // Rotate the light around the center of the scene
        let light_loc = {
            let x = 3.0 * self.light_t.cos();
            let z = 3.0 * self.light_t.sin();
            [x as f32, 5.0, z as f32]
        };

        // Render the scene from the light's point of view into depth buffer
        // ===============================================================================
        {
            // Orthographic projection used to demostrate a far-away light source
            let w = 4.0;
            let depth_projection_matrix: cgmath::Matrix4<f32> = cgmath::ortho(-w, w, -w, w, -10.0, 20.0);
            let view_center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
            let view_up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
            let depth_view_matrix = cgmath::Matrix4::look_at_rh(light_loc.into(), view_center, view_up);

            let mut draw_params: glium::draw_parameters::DrawParameters<'_> = Default::default();
            draw_params.depth = glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            };
            draw_params.backface_culling = glium::BackfaceCullingMode::CullClockwise;

            // Write depth to shadow map texture
            let mut target = glium::framebuffer::SimpleFrameBuffer::depth_only(display, &self.shadow_texture).unwrap();
            target.clear_color(1.0, 1.0, 1.0, 1.0);
            target.clear_depth(1.0);

            // Draw each model
            for md in &mut self.model_data {
                let depth_mvp = depth_projection_matrix * depth_view_matrix * md.model_matrix;
                md.depth_mvp = depth_mvp;

                let uniforms = uniform! {
                    depth_mvp: Into::<[[f32; 4]; 4]>::into(depth_mvp),
                };

                target.draw(
                    &self.model_vertex_buffer,
                    &self.model_index_buffer,
                    &self.shadow_map_shaders,
                    &uniforms,
                    &draw_params,
                ).unwrap();
            }
        }

        // Render the scene from the camera's point of view
        // ===============================================================================
        let (width, height) = display.get_framebuffer_dimensions();
        let screen_ratio = (width / height) as f32;
        let perspective_matrix: cgmath::Matrix4<f32> = cgmath::perspective(cgmath::Deg(45.0), screen_ratio, 0.0001, 100.0);
        let camera_x = 3.0 * self.camera_t.cos();
        let camera_z = 3.0 * self.camera_t.sin();
        let view_eye: cgmath::Point3<f32> = cgmath::Point3::new(camera_x as f32, 2.0, camera_z as f32);
        let view_center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
        let view_up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
        let view_matrix: cgmath::Matrix4<f32> = cgmath::Matrix4::look_at_rh(view_eye, view_center, view_up);

        let bias_matrix: cgmath::Matrix4<f32> = [
            [0.5, 0.0, 0.0, 0.0],
            [0.0, 0.5, 0.0, 0.0],
            [0.0, 0.0, 0.5, 0.0],
            [0.5, 0.5, 0.5, 1.0],
        ].into();

        let mut draw_params: glium::draw_parameters::DrawParameters<'_> = Default::default();
        draw_params.depth = glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLessOrEqual,
            write: true,
            ..Default::default()
        };
        draw_params.backface_culling = glium::BackfaceCullingMode::CullCounterClockwise;
        draw_params.blend = glium::Blend::alpha_blending();

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        // Draw each model
        for md in &self.model_data {
            let mvp = perspective_matrix * view_matrix * md.model_matrix;
            let depth_bias_mvp = bias_matrix * md.depth_mvp;

            let uniforms = uniform! {
                light_loc: light_loc,
                perspective_matrix: Into::<[[f32; 4]; 4]>::into(perspective_matrix),
                view_matrix: Into::<[[f32; 4]; 4]>::into(view_matrix),
                model_matrix: Into::<[[f32; 4]; 4]>::into(md.model_matrix),
                model_color: md.color,

                mvp: Into::<[[f32;4];4]>::into(mvp),
                depth_bias_mvp: Into::<[[f32;4];4]>::into(depth_bias_mvp),
                shadow_map: glium::uniforms::Sampler::new(&self.shadow_texture)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .depth_texture_comparison(Some(glium::uniforms::DepthTextureComparison::LessOrEqual)),
            };

            target.draw(
                &self.model_vertex_buffer,
                &self.model_index_buffer,
                &self.render_shaders,
                &uniforms,
                &draw_params,
            ).unwrap();
        }

        {
            let uniforms = uniform! {
                tex: glium::uniforms::Sampler::new(&self.shadow_texture)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
            };
            target.clear_depth(1.0);
            target
                .draw(
                    &self.debug_vertex_buffer,
                    &self.debug_index_buffer,
                    &self.debug_shadow_map_shaders,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        }
        target.finish().unwrap();
    }

    fn update(&mut self) {
        let elapsed_dur = self.start.elapsed();
        let secs = (elapsed_dur.as_secs() as f64) + (elapsed_dur.subsec_nanos() as f64) * 1e-9;
        self.start = Instant::now();
        if self.camera_rotating { self.camera_t += secs * 0.7; }
        if self.light_rotating { self.light_t += secs * 0.7; }
    }
}

fn main() {
    println!("This example demonstrates real-time shadow mapping. Press C to toggle camera");
    println!("rotation; press L to toggle light rotation.");
    State::<Application>::run_loop();
}
