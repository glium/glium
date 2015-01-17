#![feature(plugin)]

#[plugin]
extern crate glium_macros;
extern crate glutin;
extern crate glium;
#[cfg(feature = "cgmath")]
extern crate cgmath;
#[cfg(feature = "image")]
extern crate image;

use glium::Surface;
use glium::DisplayBuild;
#[cfg(feature = "cgmath")]
use cgmath::FixedArray;
use std::io::BufReader;

#[cfg(not(all(feature = "cgmath", feature = "image")))]
fn main() {
    println!("This example requires the `cgmath` and `image` features to be enabled");
}

#[cfg(all(feature = "cgmath", feature = "image"))]
fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_dimensions(800, 500)
        .with_title(format!("Glium Deferred Example"))
        .build_glium()
        .unwrap();

    let image = image::load(BufReader::new(include_bytes!("../tests/fixture/opengl.png")), image::PNG).unwrap();
    let opengl_texture = glium::texture::Texture2d::new(&display, image);

    let floor_vertex_buffer = {
        #[vertex_format]
        #[derive(Copy)]
        struct Vertex {
            position: [f32; 4],
            normal: [f32; 4],
            texcoord: [f32; 2]
        }
        
        glium::VertexBuffer::new(&display,
            vec![
                Vertex { position: [-1.0, 0.0, -1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                Vertex { position: [1.0, 0.0, -1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
                Vertex { position: [1.0, 0.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
                Vertex { position: [-1.0, 0.0, 1.0, 1.0], normal: [0.0, 1.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
            ]
        )
    };

    let floor_index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TrianglesList(vec![0u16, 1, 2, 0, 2, 3]));

    let quad_vertex_buffer = {
        #[vertex_format]
        #[derive(Copy)]
        struct Vertex {
            position: [f32; 4],
            texcoord: [f32; 2]
        }
        
        glium::VertexBuffer::new(&display,
            vec![
                Vertex { position: [0.0, 0.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                Vertex { position: [800.0, 0.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
                Vertex { position: [800.0, 500.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
                Vertex { position: [0.0, 500.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
            ]
        )
    };

    let quad_index_buffer = glium::IndexBuffer::new(&display,
        glium::index_buffer::TrianglesList(vec![0u16, 1, 2, 0, 2, 3]));

    // compiling shaders and linking them together
    let prepass_program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 130

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
            #version 130
            
            uniform sampler2D texture;

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
                output3 = texture2D(texture, frag_texcoord);
                output4 = vec4(1.0, 0.0, 1.0, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    let lighting_program = glium::Program::from_source(&display,
        // vertex shader
        "
            #version 130

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
            #version 130
            
            uniform sampler2D position_texture;
            uniform sampler2D normal_texture;
            uniform vec4 light_position;
            uniform vec3 light_color;
            uniform vec3 light_attenuation;
            uniform float light_radius;

            smooth in vec2 frag_texcoord;

            out vec4 frag_output;

            void main() {
                vec4 position = texture2D(position_texture, frag_texcoord);
                vec4 normal = texture2D(normal_texture, frag_texcoord);
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
            #version 130

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
            #version 130

            uniform sampler2D decal_texture;
            uniform sampler2D lighting_texture;

            smooth in vec2 frag_texcoord;

            out vec4 frag_output;

            void main() {
                vec4 lighting_value = texture2D(lighting_texture, frag_texcoord);
                frag_output = vec4(texture2D(decal_texture, frag_texcoord).rgb * lighting_value.rgb, 1.0);
            }
        ",

        // geometry shader
        None)
        .unwrap();

    // creating the uniforms structure
    #[uniforms]
    #[derive(Copy)]
    struct PrepassUniforms<'a> {
        perspective_matrix: [[f32; 4]; 4],
        view_matrix: [[f32; 4]; 4],
        model_matrix: [[f32; 4]; 4],
        texture: &'a glium::texture::Texture2d
    }

    #[uniforms]
    #[derive(Copy)]
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
    #[derive(Copy)]
    struct CompositionUniforms<'a> {
        matrix: [[f32; 4]; 4],
        decal_texture: &'a glium::texture::Texture2d,
        lighting_texture: &'a glium::texture::Texture2d
    }

    struct Light<'a> {
        position: [f32; 4],
        color: [f32; 3],
        attenuation: [f32; 3],
        radius: f32
    }

    let texture1 = glium::texture::Texture2d::new_empty(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, 800, 500);
    let texture2 = glium::texture::Texture2d::new_empty(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, 800, 500);
    let texture3 = glium::texture::Texture2d::new_empty(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, 800, 500);
    let texture4 = glium::texture::Texture2d::new_empty(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, 800, 500);
    let depthtexture = glium::texture::DepthTexture2d::new_empty(&display, glium::texture::DepthFormat::F32, 800, 500);
    let output = &[("output1", &texture1), ("output2", &texture2), ("output3", &texture3), ("output4", &texture4)];
    let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(&display, output, &depthtexture);

    let light_texture = glium::texture::Texture2d::new_empty(&display, glium::texture::UncompressedFloatFormat::F32F32F32F32, 800, 500);
    let mut light_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display, &light_texture, &depthtexture);

    let ortho_matrix: cgmath::Matrix4<f32> = cgmath::ortho(0.0, 800.0, 0.0, 500.0, -1.0, 1.0);
    let fixed_ortho_matrix = ortho_matrix.as_fixed();

    let perspective_matrix: cgmath::Matrix4<f32> = cgmath::perspective(cgmath::deg(45.0), 1.333, 0.0001, 100.0);
    let fixed_perspective_matrix = perspective_matrix.as_fixed();
    let view_eye: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 2.0, -2.0);
    let view_center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
    let view_up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 1.0, 0.0);
    let view_matrix: cgmath::Matrix4<f32> = cgmath::Matrix4::look_at(&view_eye, &view_center, &view_up);
    let fixed_view_matrix = view_matrix.as_fixed();
    let model_matrix: cgmath::Matrix4<f32> = cgmath::Matrix4::identity();
    let fixed_model_matrix = model_matrix.as_fixed();

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
    // each cycle will draw once
    'main: loop {
        use std::io::timer;
        use std::time::Duration;

        // prepass
        let uniforms = PrepassUniforms {
            perspective_matrix: *fixed_perspective_matrix,
            view_matrix: *fixed_view_matrix,
            model_matrix: *fixed_model_matrix,
            texture: &opengl_texture
        };
        framebuffer.draw(&floor_vertex_buffer, &floor_index_buffer, &prepass_program, &uniforms, &std::default::Default::default()).unwrap();

        // lighting
        let draw_params = glium::DrawParameters {
            //depth_function: glium::DepthFunction::IfLessOrEqual,
            blending_function: Some(glium::BlendingFunction::Addition{
                source: glium::LinearBlendingFactor::One,
                destination: glium::LinearBlendingFactor::One
            }),
            .. std::default::Default::default()
        };
        light_buffer.clear_color(0.0, 0.0, 0.0, 0.0);
        for light in lights.iter() {
            let uniforms = LightingUniforms {
                matrix: *fixed_ortho_matrix,
                position_texture: &texture1,
                normal_texture: &texture2,
                light_position: light.position,
                light_attenuation: light.attenuation,
                light_color: light.color,
                light_radius: light.radius
            };
            light_buffer.draw(&quad_vertex_buffer, &quad_index_buffer, &lighting_program, &uniforms, &draw_params).unwrap();
        }

        // composition
        let uniforms = CompositionUniforms {
            matrix: *fixed_ortho_matrix,
            decal_texture: &texture3,
            lighting_texture: &light_texture
        };
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&quad_vertex_buffer, &quad_index_buffer, &composition_program, &uniforms, &std::default::Default::default()).unwrap();
        target.finish();

        // sleeping for some time in order not to use up too much CPU
        timer::sleep(Duration::milliseconds(17));

        // polling and handling the events received by the window
        for event in display.poll_events().into_iter() {
            match event {
                glutin::Event::Closed => break 'main,
                _ => ()
            }
        }
    }
}
