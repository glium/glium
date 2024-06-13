#[macro_use]
extern crate glium;
use glium::Surface;

mod support;
use support::view_matrix;

fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Glium tutorial #14")
        .build(&event_loop);

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        normal: [f32; 3],
        tex_coords: [f32; 2],
    }
    implement_vertex!(Vertex, position, normal, tex_coords);
    let shape = glium::vertex::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0] },
        Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0] },
    ]).unwrap();

    let image = image::load(std::io::Cursor::new(&include_bytes!("../book/resources/tuto-14-diffuse.jpg")),
                            image::ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let diffuse_texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let image = image::load(std::io::Cursor::new(&include_bytes!("../book/resources/tuto-14-normal.png")),
                            image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let normal_map = glium::texture::Texture2d::new(&display, image).unwrap();


    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec3 normal;
        in vec2 tex_coords;

        out vec3 v_normal;
        out vec3 v_position;
        out vec2 v_tex_coords;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            v_tex_coords = tex_coords;
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * vec4(position, 1.0);
            v_position = gl_Position.xyz / gl_Position.w;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        in vec3 v_position;
        in vec2 v_tex_coords;

        out vec4 color;

        uniform vec3 u_light;
        uniform sampler2D diffuse_tex;
        uniform sampler2D normal_tex;

        const vec3 specular_color = vec3(1.0, 1.0, 1.0);

        mat3 cotangent_frame(vec3 normal, vec3 pos, vec2 uv) {
            vec3 dp1 = dFdx(pos);
            vec3 dp2 = dFdy(pos);
            vec2 duv1 = dFdx(uv);
            vec2 duv2 = dFdy(uv);

            vec3 dp2perp = cross(dp2, normal);
            vec3 dp1perp = cross(normal, dp1);
            vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
            vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;

            float invmax = inversesqrt(max(dot(T, T), dot(B, B)));
            return mat3(T * invmax, B * invmax, normal);
        }

        void main() {
            vec3 diffuse_color = texture(diffuse_tex, v_tex_coords).rgb;
            vec3 ambient_color = diffuse_color * 0.1;

            vec3 v_normal_unit = normalize(v_normal);
            vec3 normal_map = texture(normal_tex, v_tex_coords).rgb;
            mat3 tbn = cotangent_frame(v_normal_unit, -v_position, v_tex_coords);
            vec3 real_normal = normalize(tbn * -(normal_map * 2.0 - 1.0));

            float diffuse = max(dot(real_normal, normalize(u_light)), 0.0);

            vec3 camera_dir = normalize(-v_position);
            vec3 half_direction = normalize(normalize(u_light) + camera_dir);
            float specular = pow(max(dot(half_direction, real_normal), 0.0), 16.0);

            color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src,
                                            None).unwrap();
    let start = std::time::Instant::now();

    #[allow(deprecated)]
    event_loop.run(move |ev, window_target| {
        match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                },
                // We now need to render everyting in response to a RedrawRequested event due to the animation
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let mut target = display.draw();
                    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

                    let t = (std::time::Instant::now() - start).as_secs_f32() * 2.0;
                    let ang = t.sin();
                    let (c, s) = (ang.cos(), ang.sin());
                    let model = [
                        [  c, 0.0,   s, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [ -s, 0.0,   c, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32]
                    ];

                    let view = view_matrix(&[0.5, 0.2, -3.0], &[-0.5, -0.2, 3.0], &[0.0, 1.0, 0.0]);

                    let perspective = {
                        let (width, height) = target.get_dimensions();
                        let aspect_ratio = height as f32 / width as f32;

                        let fov: f32 = 3.141592 / 3.0;
                        let zfar = 1024.0;
                        let znear = 0.1;

                        let f = 1.0 / (fov / 2.0).tan();

                        [
                            [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
                            [         0.0         ,     f ,              0.0              ,   0.0],
                            [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
                            [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
                        ]
                    };

                    let light = [1.4, 0.4, 0.7f32];

                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        .. Default::default()
                    };

                    target.draw(&shape, glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
                                &uniform! { model: model, view: view, perspective: perspective,
                                            u_light: light, diffuse_tex: &diffuse_texture, normal_tex: &normal_map },
                                &params).unwrap();
                    target.finish().unwrap();
                },
                // Because glium doesn't know about windows we need to resize the display
                // when the window's size has changed.
                glium::winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                },
                _ => (),
            },
            // By requesting a redraw in response to a AboutToWait event we get continuous rendering.
            // For applications that only change due to user input you could remove this handler.
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw();
            },
            _ => (),
        }
    })
    .unwrap();
}
