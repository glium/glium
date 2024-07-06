#[macro_use]
extern crate glium;

use glium::{Display, Surface};
use glutin::surface::WindowSurface;
use std::io::Cursor;
use support::{ApplicationContext, State};

mod support;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

struct Application {
    pub opengl_texture: glium::texture::CompressedTexture2d,
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub program: glium::Program,
    pub camera: support::camera::CameraState,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium displacement_mapping example";

    fn new(display: &Display<WindowSurface>) -> Self {
        let image = image::load(
            Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = glium::texture::CompressedTexture2d::new(display, image).unwrap();

        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            glium::VertexBuffer::new(
                display,
                &[
                    Vertex {
                        position: [-0.5, 0.5, 0.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, 0.5, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [0.5, 0.5, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.0],
                        tex_coords: [1.0, 0.0],
                    },
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                ],
            )
            .unwrap()
        };

        // compiling shaders and linking them together
        let program = glium::Program::new(
            display,
            glium::program::SourceCode {
                vertex_shader: "
                    #version 400

                    in vec3 position;
                    in vec2 tex_coords;

                    out vec3 v_position;
                    out vec3 v_normal;
                    out vec2 v_tex_coords;

                    void main() {
                        v_position = position;
                        v_normal = vec3(0.0, 0.0, -1.0);
                        v_tex_coords = tex_coords;
                    }
                ",
                tessellation_control_shader: Some(
                    "
                    #version 400

                    layout(vertices = 3) out;

                    in vec3 v_position[];
                    in vec3 v_normal[];
                    in vec2 v_tex_coords[];

                    out vec3 tc_position[];
                    out vec3 tc_normal[];
                    out vec2 tc_tex_coords[];

                    uniform float inner_level;
                    uniform float outer_level;

                    void main() {
                        tc_position[gl_InvocationID] = v_position[gl_InvocationID];
                        tc_normal[gl_InvocationID]   = v_normal[gl_InvocationID];
                        tc_tex_coords[gl_InvocationID] = v_tex_coords[gl_InvocationID];

                        gl_TessLevelOuter[0] = outer_level;
                        gl_TessLevelOuter[1] = outer_level;
                        gl_TessLevelOuter[2] = outer_level;
                        gl_TessLevelOuter[3] = outer_level;
                        gl_TessLevelInner[0] = inner_level;
                        gl_TessLevelInner[1] = inner_level;
                    }
                ",
                ),
                tessellation_evaluation_shader: Some(
                    "
                    #version 400

                    layout(triangles, equal_spacing, ccw) in;

                    in vec3 tc_position[];
                    in vec3 tc_normal[];
                    in vec2 tc_tex_coords[];

                    out vec4 te_position;
                    out vec3 te_normal;
                    out vec2 te_tex_coords;

                    uniform mat4 projection_matrix;
                    uniform mat4 view_matrix;

                    uniform sampler2D height_texture;
                    uniform float elevation;

                    void main() {
                        vec3 pos = gl_TessCoord.x * tc_position[0] +
                                gl_TessCoord.y * tc_position[1] +
                                gl_TessCoord.z * tc_position[2];

                        vec3 normal = normalize(gl_TessCoord.x * tc_normal[0] +
                                                gl_TessCoord.y * tc_normal[1] +
                                                gl_TessCoord.z * tc_normal[2]);

                        vec2 tex_coords = gl_TessCoord.x * tc_tex_coords[0] +
                                        gl_TessCoord.y * tc_tex_coords[1] +
                                        gl_TessCoord.z * tc_tex_coords[2];

                        float height = length(texture(height_texture, tex_coords));
                        pos += normal * (height * elevation);

                        te_position = projection_matrix * view_matrix * vec4(pos, 1.0);
                        te_normal = vec3(view_matrix * vec4(normal, 1.0)).xyz;
                        te_tex_coords = tex_coords;
                    }
                ",
                ),
                geometry_shader: Some(
                    "
                    #version 400

                    layout(triangles) in;
                    layout(triangle_strip, max_vertices = 3) out;

                    uniform mat4 view_matrix;

                    in vec4 te_position[3];
                    in vec3 te_normal[3];
                    in vec2 te_tex_coords[3];

                    out vec3 g_normal;
                    out vec2 g_tex_coords;

                    void main() {
                        g_normal = te_normal[0];
                        g_tex_coords = te_tex_coords[0];
                        gl_Position = te_position[0];
                        EmitVertex();

                        g_normal = te_normal[1];
                        g_tex_coords = te_tex_coords[1];
                        gl_Position = te_position[1];
                        EmitVertex();

                        g_normal = te_normal[2];
                        g_tex_coords = te_tex_coords[2];
                        gl_Position = te_position[2];
                        EmitVertex();

                        EndPrimitive();
                    }
                ",
                ),
                fragment_shader: "
                    #version 400

                    in vec3 g_normal;
                    in vec2 g_tex_coords;

                    out vec4 o_color;

                    uniform sampler2D color_texture;

                    const vec3 LIGHT = vec3(-0.2, 0.1, -0.8);

                    void main() {
                        float lum = max(dot(normalize(g_normal), normalize(LIGHT)), 0.0);
                        vec3 tex_color = texture(color_texture, g_tex_coords).rgb;
                        vec3 color = (0.6 + 0.4 * lum) * tex_color;
                        o_color = vec4(color, 1);
                    }
                ",
            },
        )
        .unwrap();

        let camera = support::camera::CameraState::new();
        Self {
            opengl_texture,
            vertex_buffer,
            program,
            camera,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let mut frame = display.draw();
        // building the uniforms
        let uniforms = uniform! {
            inner_level: 64.0f32,
            outer_level: 64.0f32,
            projection_matrix: self.camera.get_perspective(),
            view_matrix: self.camera.get_view(),
            height_texture: &self.opengl_texture,
            elevation: 0.3f32,
            color_texture: &self.opengl_texture
        };

        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame
            .draw(
                &self.vertex_buffer,
                &glium::index::NoIndices(glium::index::PrimitiveType::Patches {
                    vertices_per_patch: 3,
                }),
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();
    }

    fn handle_window_event(&mut self, event: &glium::winit::event::WindowEvent, _window: &glium::winit::window::Window) {
        self.camera.process_input(&event);
    }

    fn update(&mut self) {
        self.camera.update();
    }
}

fn main() {
    State::<Application>::run_loop();
}
