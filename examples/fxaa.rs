#[macro_use]
extern crate glium;

#[allow(unused_imports)]
use glium::{glutin, Surface};

mod support;

mod fxaa {
    use glium::{self, Surface};
    use glium::backend::Facade;
    use glium::backend::Context;
    use glium::framebuffer::SimpleFrameBuffer;

    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct FxaaSystem {
        context: Rc<Context>,
        vertex_buffer: glium::VertexBuffer<SpriteVertex>,
        index_buffer: glium::IndexBuffer<u16>,
        program: glium::Program,
        target_color: RefCell<Option<glium::texture::Texture2d>>,
        target_depth: RefCell<Option<glium::framebuffer::DepthRenderBuffer>>,
    }

    #[derive(Copy, Clone)]
    struct SpriteVertex {
        position: [f32; 2],
        i_tex_coords: [f32; 2],
    }

    implement_vertex!(SpriteVertex, position, i_tex_coords);

    impl FxaaSystem {
        pub fn new<F: ?Sized>(facade: &F) -> FxaaSystem where F: Facade + Clone {
            FxaaSystem {
                context: facade.get_context().clone(),

                vertex_buffer: glium::VertexBuffer::new(facade,
                    &[
                        SpriteVertex { position: [-1.0, -1.0], i_tex_coords: [0.0, 0.0] },
                        SpriteVertex { position: [-1.0,  1.0], i_tex_coords: [0.0, 1.0] },
                        SpriteVertex { position: [ 1.0,  1.0], i_tex_coords: [1.0, 1.0] },
                        SpriteVertex { position: [ 1.0, -1.0], i_tex_coords: [1.0, 0.0] }
                    ]
                ).unwrap(),

                index_buffer: glium::index::IndexBuffer::new(facade,
                    glium::index::PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap(),

                program: program!(facade,
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
    }

    pub fn draw<T, F, R>(system: &FxaaSystem, target: &mut T, enabled: bool, mut draw: F)
                         -> R where T: Surface, F: FnMut(&mut SimpleFrameBuffer) -> R
    {
        let target_dimensions = target.get_dimensions();

        let mut target_color = system.target_color.borrow_mut();
        let mut target_depth = system.target_depth.borrow_mut();

        {
            let clear = if let &Some(ref tex) = &*target_color {
                tex.get_width() != target_dimensions.0 ||
                    tex.get_height().unwrap() != target_dimensions.1
            } else {
                false
            };
            if clear { *target_color = None; }
        }

        {
            let clear = if let &Some(ref tex) = &*target_depth {
                tex.get_dimensions() != target_dimensions
            } else {
                false
            };
            if clear { *target_depth = None; }
        }

        if target_color.is_none() {
            let texture = glium::texture::Texture2d::empty(&system.context,
                                                           target_dimensions.0 as u32,
                                                           target_dimensions.1 as u32).unwrap();
            *target_color = Some(texture);
        }
        let target_color = target_color.as_ref().unwrap();

        if target_depth.is_none() {
            let texture = glium::framebuffer::DepthRenderBuffer::new(&system.context,
                                                                      glium::texture::DepthFormat::I24,
                                                                      target_dimensions.0 as u32,
                                                                      target_dimensions.1 as u32).unwrap();
            *target_depth = Some(texture);
        }
        let target_depth = target_depth.as_ref().unwrap();

        let output = draw(&mut SimpleFrameBuffer::with_depth_buffer(&system.context, target_color,
                                                                                     target_depth).unwrap());

        let uniforms = uniform! {
            tex: &*target_color,
            enabled: if enabled { 1i32 } else { 0i32 },
            resolution: (target_dimensions.0 as f32, target_dimensions.1 as f32)
        };
        
        target.draw(&system.vertex_buffer, &system.index_buffer, &system.program, &uniforms,
                    &Default::default()).unwrap();

        output
    }
}

fn main() {
    println!("This example demonstrates FXAA. Is is an anti-aliasing technique done at the \
              post-processing stage. This example draws the teapot to a framebuffer and then \
              copies from the texture to the main framebuffer by applying a filter to it.\n\
              You can use the space bar to switch fxaa on and off.");

    // building the display, ie. the main object
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex and index buffers
    let vertex_buffer = support::load_wavefront(&display, include_bytes!("support/teapot.obj"));

    // the program
    let program = program!(&display,
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
    ).unwrap();

    //
    let mut camera = support::camera::CameraState::new();
    let fxaa = fxaa::FxaaSystem::new(&display);
    let mut fxaa_enabled = true;
    
    // the main loop
    support::start_loop(event_loop, move |events| {
        camera.update();

        // building the uniforms
        let uniforms = uniform! {
            persp_matrix: camera.get_perspective(),
            view_matrix: camera.get_view(),
        };

        // draw parameters
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        // drawing a frame
        let mut target = display.draw();
        fxaa::draw(&fxaa, &mut target, fxaa_enabled, |target| {
            target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
            target.draw(&vertex_buffer,
                        &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                        &program, &uniforms, &params).unwrap();
        });
        target.finish().unwrap();

        let mut action = support::Action::Continue;

        // polling and handling the events received by the window
        for event in events {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => {
                    camera.process_input(&event);
                    match event {
                        glutin::event::WindowEvent::CloseRequested => action = support::Action::Stop,
                        glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.state {
                            glutin::event::ElementState::Pressed => match input.virtual_keycode {
                                Some(glutin::event::VirtualKeyCode::Escape) => action = support::Action::Stop,
                                Some(glutin::event::VirtualKeyCode::Space) => {
                                    fxaa_enabled = !fxaa_enabled;
                                    println!("FXAA is now {}", if fxaa_enabled { "enabled" } else { "disabled" });
                                },
                                _ => (),
                            },
                            _ => (),
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        };

        action
    });
}
