#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
#[macro_use]
extern crate rental;

use glium::index::PrimitiveType;
#[allow(unused_imports)]
use glium::{glutin, Surface};
use std::io::Cursor;

mod support;

pub struct Dt {
    depthtexture: glium::texture::DepthTexture2d,
    textures: [glium::texture::Texture2d; 4],
    light_texture: glium::texture::Texture2d,
}

rental! {
    mod my_rentals {
        use super::Dt;
        #[rental]
        pub struct Data {
            dt: Box<Dt>,
            buffs: (glium::framebuffer::MultiOutputFrameBuffer<'dt>, glium::framebuffer::SimpleFrameBuffer<'dt>, &'dt Dt),
        }
    }
}

fn main() {
    use cgmath::SquareMatrix;

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size((800, 500).into())
        .with_title("Glium Deferred Example");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let image = image::load(Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]), image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let floor_vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 4],
            normal: [f32; 4],
            texcoord: [f32; 2]
        }

        implement_vertex!(Vertex, position, normal, texcoord);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-1.0, 0.0, -1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
                Vertex { position: [1.0, 0.0, -1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                Vertex { position: [1.0, 0.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
                Vertex { position: [-1.0, 0.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
            ]
        ).unwrap()
    };

    let floor_index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                                     &[0u16, 1, 2, 0, 2, 3]).unwrap();

    let quad_vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 4],
            texcoord: [f32; 2]
        }

        implement_vertex!(Vertex, position, texcoord);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [0.0, 0.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                Vertex { position: [800.0, 0.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
                Vertex { position: [800.0, 500.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.0, 500.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
            ]
        ).unwrap()
    };

    let quad_index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                                    &[0u16, 1, 2, 0, 2, 3]).unwrap();

    // compiling shaders and linking them together
    let prepass_program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 140

            uniform mat4 perspective_matrix;
            uniform mat4 view_matrix;
            uniform mat4 model_matrix;

            in vec4 position;
            in vec4 normal;
            in vec2 texcoord;

            smooth out vec4 frag_position;
            smooth out vec4 frag_normal;
            smooth out vec2 frag_texcoord;

            void main() {
                frag_position = model_matrix * position;
                frag_normal = model_matrix * normal;
                frag_texcoord = texcoord;
                gl_Position = perspective_matrix * view_matrix * frag_position;
            }
        ",

        // fragment shader
        "
            #version 140

            uniform sampler2D tex;

            smooth in vec4 frag_position;
            smooth in vec4 frag_normal;
            smooth in vec2 frag_texcoord;

            out vec4 output1;
            out vec4 output2;
            out vec4 output3;
            out vec4 output4;

            void main() {
                output1 = vec4(frag_position);
                output2 = vec4(frag_normal);
                output3 = texture(tex, frag_texcoord);
                output4 = vec4(1.0, 0.0, 1.0, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    let lighting_program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 140

            uniform mat4 matrix;

            in vec4 position;
            in vec2 texcoord;

            smooth out vec2 frag_texcoord;

            void main() {
                gl_Position = matrix * position;
                frag_texcoord = texcoord;
            }
        ",

        // fragment shader
        "
            #version 140

            uniform sampler2D position_texture;
            uniform sampler2D normal_texture;
            uniform vec4 light_position;
            uniform vec3 light_color;
            uniform vec3 light_attenuation;
            uniform float light_radius;

            smooth in vec2 frag_texcoord;

            out vec4 frag_output;

            void main() {
                vec4 position = texture(position_texture, frag_texcoord);
                vec4 normal = texture(normal_texture, frag_texcoord);
                vec3 light_vector = light_position.xyz - position.xyz;
                float light_distance = abs(length(light_vector));
                vec3 normal_vector = normalize(normal.xyz);
                float diffuse = max(dot(normal_vector, light_vector), 0.0);
                if (diffuse > 0.0) {
                    float attenuation_factor = 1.0 / (
                        light_attenuation.x +
                        (light_attenuation.y * light_distance) +
                        (light_attenuation.z * light_distance * light_distance)
                    );
                    attenuation_factor *= (1.0 - pow((light_distance / light_radius), 2.0));
		    attenuation_factor = max(attenuation_factor, 0.0);
                    diffuse *= attenuation_factor;

                }
                frag_output = vec4(light_color * diffuse, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // compiling shaders and linking them together
    let composition_program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 140

            uniform mat4 matrix;

            in vec4 position;
            in vec2 texcoord;

            smooth out vec2 frag_texcoord;

            void main() {
                frag_texcoord = texcoord;
                gl_Position = matrix * position;
            }
        ",

        // fragment shader
        "
            #version 140

            uniform sampler2D decal_texture;
            uniform sampler2D lighting_texture;

            smooth in vec2 frag_texcoord;

            out vec4 frag_output;

            void main() {
                vec4 lighting_value = texture(lighting_texture, frag_texcoord);
                frag_output = vec4(texture(decal_texture, frag_texcoord).rgb * lighting_value.rgb, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    /*// creating the uniforms structure
    #[uniforms]
    #[derive(Copy, Clone)]
    struct PrepassUniforms<'a> {
        perspective_matrix: [[f32; 4]; 4],
        view_matrix: [[f32; 4]; 4],
        model_matrix: [[f32; 4]; 4],
        texture: &'a glium::texture::Texture2d
    }

    #[uniforms]
    #[derive(Copy, Clone)]
    struct LightingUniforms<'a> {
        matrix: [[f32; 4]; 4],
        position_texture: &'a glium::texture::Texture2d,
        normal_texture: &'a glium::texture::Texture2d,
        light_position: [f32; 4],
        light_color: [f32; 3],
        light_attenuation: [f32; 3],
        light_radius: f32
    }

    #[uniforms]
    #[derive(Copy, Clone)]
    struct CompositionUniforms<'a> {
        matrix: [[f32; 4]; 4],
        decal_texture: &'a glium::texture::Texture2d,
        lighting_texture: &'a glium::texture::Texture2d
    }*/

    struct Light {
        position: [f32; 4],
        color: [f32; 3],
        attenuation: [f32; 3],
        radius: f32
    }

    let texture1 = glium::texture::Texture2d::empty_with_format(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, glium::texture::MipmapsOption::NoMipmap, 800, 500).unwrap();
    let texture2 = glium::texture::Texture2d::empty_with_format(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, glium::texture::MipmapsOption::NoMipmap, 800, 500).unwrap();
    let texture3 = glium::texture::Texture2d::empty_with_format(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, glium::texture::MipmapsOption::NoMipmap, 800, 500).unwrap();
    let texture4 = glium::texture::Texture2d::empty_with_format(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, glium::texture::MipmapsOption::NoMipmap, 800, 500).unwrap();
    let depthtexture = glium::texture::DepthTexture2d::empty_with_format(&display, glium::texture::DepthFormat::F32, glium::texture::MipmapsOption::NoMipmap, 800, 500).unwrap();
    let light_texture = glium::texture::Texture2d::empty_with_format(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, glium::texture::MipmapsOption::NoMipmap, 800, 500).unwrap();

    let mut tenants = my_rentals::Data::new(
        Box::new(Dt {
            depthtexture,
            textures: [texture1, texture2, texture3, texture4],
            light_texture,
        }),
        |dt| {
            let output = [("output1", &dt.textures[0]), ("output2", &dt.textures[1]), ("output3", &dt.textures[2]), ("output4", &dt.textures[3])];
            let framebuffer = glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(&display, output.into_iter().cloned(), &dt.depthtexture).unwrap();
            let light_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display, &dt.light_texture, &dt.depthtexture).unwrap();
            (framebuffer, light_buffer, dt)
        }
    );


    let ortho_matrix: cgmath::Matrix4<f32> = cgmath::ortho(0.0, 800.0, 0.0, 500.0, -1.0, 1.0);

    let perspective_matrix: cgmath::Matrix4<f32> = cgmath::perspective(cgmath::Deg(45.0), 1.333, 0.0001, 100.0);
    let view_eye: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 2.0, -2.0);
    let view_center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
    let view_up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
    let view_matrix: cgmath::Matrix4<f32> = cgmath::Matrix4::look_at(view_eye, view_center, view_up);
    let model_matrix: cgmath::Matrix4<f32> = cgmath::Matrix4::identity();

    let lights = [
        Light {
            position: [1.0, 1.0, 1.0, 1.0],
            attenuation: [0.8, 0.00125, 0.0000001],
            color: [1.0, 0.0, 0.0],
            radius: 1.5
        },
        Light {
            position: [0.0, 1.0, 0.0, 1.0],
            attenuation: [0.8, 0.00125, 0.0000001],
            color: [0.0, 1.0, 0.0],
            radius: 1.5
        },
        Light {
            position: [0.0, 1.0, 1.0, 1.0],
            attenuation: [0.8, 0.00125, 0.0000001],
            color: [0.0, 0.0, 1.0],
            radius: 1.5
        },
        Light {
            position: [1.0, 1.0, 0.0, 1.0],
            attenuation: [0.8, 0.00125, 0.0000001],
            color: [1.0, 1.0, 0.0],
            radius: 1.5
        }
    ];

    // the main loop
    support::start_loop(event_loop, move |events| {
        tenants.rent_mut(|(framebuffer, light_buffer, dt)| {
            // prepass
            let uniforms = uniform! {
                perspective_matrix: Into::<[[f32; 4]; 4]>::into(perspective_matrix),
                view_matrix: Into::<[[f32; 4]; 4]>::into(view_matrix),
                model_matrix: Into::<[[f32; 4]; 4]>::into(model_matrix),
                tex: &opengl_texture
            };
            framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);
            framebuffer.draw(&floor_vertex_buffer, &floor_index_buffer, &prepass_program, &uniforms, &Default::default()).unwrap();

            // lighting
            let draw_params = glium::DrawParameters {
                //depth_function: glium::DepthFunction::IfLessOrEqual,
                blend: glium::Blend {
                    color: glium::BlendingFunction::Addition {
                        source: glium::LinearBlendingFactor::One,
                        destination: glium::LinearBlendingFactor::One
                    },
                    alpha: glium::BlendingFunction::Addition {
                        source: glium::LinearBlendingFactor::One,
                        destination: glium::LinearBlendingFactor::One
                    },
                    constant_value: (1.0, 1.0, 1.0, 1.0)
                },
                .. Default::default()
            };
            light_buffer.clear_color(0.0, 0.0, 0.0, 0.0);
            for light in lights.iter() {
                let uniforms = uniform! {
                    matrix: Into::<[[f32; 4]; 4]>::into(ortho_matrix),
                    position_texture: &dt.textures[0],
                    normal_texture: &dt.textures[1],
                    light_position: light.position,
                    light_attenuation: light.attenuation,
                    light_color: light.color,
                    light_radius: light.radius
                };
                light_buffer.draw(&quad_vertex_buffer, &quad_index_buffer, &lighting_program, &uniforms, &draw_params).unwrap();
            }

            // composition
            let uniforms = uniform! {
                matrix: Into::<[[f32; 4]; 4]>::into(ortho_matrix),
                decal_texture: &dt.textures[2],
                lighting_texture: &dt.light_texture
            };
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            target.draw(&quad_vertex_buffer, &quad_index_buffer, &composition_program, &uniforms, &Default::default()).unwrap();
            target.finish().unwrap();

            let mut action = support::Action::Continue;

            // polling and handling the events received by the window
            for event in events {
                match event {
                    glutin::event::Event::WindowEvent { event, .. } => match event {
                        glutin::event::WindowEvent::CloseRequested => action = support::Action::Stop,
                        _ => (),
                    },
                    _ => (),
                }
            };

            action
        })
    });
}
