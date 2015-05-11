#[macro_use]
extern crate glium;

use glium::Surface;
use glium::glutin;

mod support;

fn main() {
    use glium::DisplayBuild;

    println!("Occlusion query example. Press Enter to activate/deactive occlusion query. \
              You should see an FPS increase when it's activated.");

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    // building the vertex buffer for the teapot
    let teapot_vertex_buffer = support::load_wavefront(&display,
                                                       include_bytes!("support/teapot.obj"));

    // building the bounding box
    let (bounding_box_vb, bounding_box_ib) = {
        // the vertex type of the bounding box
        #[derive(Copy, Clone)]
        struct BbVertex {
            position: [f32; 3]
        }

        implement_vertex!(BbVertex, position);

        let (mut min_x, mut max_x, mut min_y, mut max_y, mut min_z, mut max_z) =
                    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        for vertex in teapot_vertex_buffer.read_if_supported().unwrap() {
            if vertex.position[0] < min_x {
                min_x = vertex.position[0];
            }
            if vertex.position[0] > max_x {
                max_x = vertex.position[0];
            }
            if vertex.position[1] < min_y {
                min_y = vertex.position[1];
            }
            if vertex.position[1] > max_y {
                max_y = vertex.position[1];
            }
            if vertex.position[2] < min_z {
                min_z = vertex.position[2];
            }
            if vertex.position[2] > max_z {
                max_z = vertex.position[2];
            }
        }

        let vb = glium::VertexBuffer::new(&display, &[
            BbVertex { position: [min_x, min_y, min_z] },
            BbVertex { position: [min_x, max_y, min_z] },
            BbVertex { position: [max_x, min_y, min_z] },
            BbVertex { position: [max_x, max_y, min_z] },
            BbVertex { position: [min_x, min_y, max_z] },
            BbVertex { position: [min_x, max_y, max_z] },
            BbVertex { position: [max_x, min_y, max_z] },
            BbVertex { position: [max_x, max_y, max_z] },
        ]);

        let ib = glium::index::IndexBuffer::new(&display,
            glium::index::TrianglesList(vec![
                0, 1, 3, 0, 3, 2,
                4, 5, 7, 4, 7, 6,
                4, 5, 1, 4, 1, 0,
                6, 7, 3, 6, 3, 2,
                0, 4, 6, 0, 6, 3,
                1, 5, 7, 1, 7, 4u16,
            ]));

        (vb, ib)
    };

    // the program of the teapot
    let teapot_program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;
                uniform vec3 translation;

                in vec3 position;
                in vec3 normal;
                out vec3 v_position;
                out vec3 v_normal;

                void main() {
                    v_position = position;
                    v_normal = normal;
                    gl_Position = persp_matrix * view_matrix * vec4(translation + v_position * 0.005, 1.0);
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
                uniform vec3 translation;

                attribute vec3 position;
                attribute vec3 normal;
                varying vec3 v_position;
                varying vec3 v_normal;

                void main() {
                    v_position = position;
                    v_normal = normal;
                    gl_Position = persp_matrix * view_matrix * vec4(translation + v_position * 0.005, 1.0);
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
                uniform lowp vec3 translation;

                attribute lowp vec3 position;
                attribute lowp vec3 normal;
                varying lowp vec3 v_position;
                varying lowp vec3 v_normal;

                void main() {
                    v_position = position;
                    v_normal = normal;
                    gl_Position = persp_matrix * view_matrix * vec4(translation + v_position * 0.005, 1.0);
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
    ).unwrap();

    // the program of the bounding box
    let boundingbox_program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;
                uniform vec3 translation;

                in vec3 position;

                void main() {
                    gl_Position = persp_matrix * view_matrix * vec4(translation + position * 0.005, 1.0);
                }
            ",

            fragment: "
                #version 140

                out vec4 f_color;

                void main() {
                    f_color = vec4(1.0, 1.0, 1.0, 1.0);
                }
            ",
        },

        110 => {
            vertex: "
                #version 110

                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;
                uniform vec3 translation;

                attribute vec3 position;

                void main() {
                    gl_Position = persp_matrix * view_matrix * vec4(translation + position * 0.005, 1.0);
                }
            ",

            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                }
            ",
        },

        100 => {
            vertex: "
                #version 100

                uniform lowp mat4 persp_matrix;
                uniform lowp mat4 view_matrix;
                uniform lowp vec3 translation;

                attribute lowp vec3 position;

                void main() {
                    gl_Position = persp_matrix * view_matrix * vec4(translation + position * 0.005, 1.0);
                }
            ",

            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                }
            ",
        },
    ).unwrap();

    // list of teapots to draw
    let mut teapots = {
        let mut teapots: Vec<((f32, f32, f32), _)> = Vec::new();

        for x in (-2 .. 3) {
            for y in (-2 .. 4) {
                for z in (-3 .. 2) {
                    teapots.push(((x as f32 * 0.7, y as f32 * 0.5, z as f32 * 0.7), None));
                }
            }
        }

        teapots
    };

    //
    let mut camera = support::camera::CameraState::new();
    let mut use_occlusion_query = true;
    
    // the main loop
    support::start_loop(|| {
        camera.update();

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        if use_occlusion_query {
            // we draw each bounding box for each teapot
            for teapot in &mut teapots {
                let query = glium::draw_parameters::
                                AnySamplesPassedQuery::new_if_supported(&display, true).unwrap();

                let uniforms = uniform! {
                    persp_matrix: camera.get_perspective(),
                    view_matrix: camera.get_view(),
                    translation: teapot.0,
                };

                target.draw(&bounding_box_vb, &bounding_box_ib,
                            &boundingbox_program, &uniforms, &glium::DrawParameters {
                                samples_passed_query: Some((&query).into()),
                                .. std::default::Default::default()
                            }).unwrap();

                teapot.1 = Some(query);
            }
        }

        // passing again but for real this time
        for teapot in &mut teapots {
            let uniforms = uniform! {
                persp_matrix: camera.get_perspective(),
                view_matrix: camera.get_view(),
                translation: teapot.0,
            };

            let mut params = glium::DrawParameters {
                depth_test: glium::DepthTest::IfLess,
                depth_write: true,
                .. std::default::Default::default()
            };

            if use_occlusion_query {
                params.condition = Some(glium::draw_parameters::ConditionalRendering {
                    query: teapot.1.as_ref().unwrap().into(),
                    wait: false,
                    per_region: true,
                });
            }

            target.draw(&teapot_vertex_buffer,
                        &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                        &teapot_program, &uniforms, &params).unwrap();
        }

        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return support::Action::Stop,
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Return)) => {
                    use_occlusion_query = !use_occlusion_query;
                },
                ev => camera.process_input(&ev),
            }
        }

        support::Action::Continue
    });
}
