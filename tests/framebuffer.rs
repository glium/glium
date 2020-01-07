#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn no_depth_buffer_depth_test() {
    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::texture::Texture2d::empty_with_format(&display,
                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                            glium::texture::MipmapsOption::NoMipmap, 128, 128).unwrap();
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();

    let parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            .. Default::default()
        },
        .. Default::default()
    };

    match framebuffer.draw(&vertex_buffer, &index_buffer, &program,
                           &glium::uniforms::EmptyUniforms, &parameters)
    {
        Err(glium::DrawError::NoDepthBuffer) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error(None);
}

#[test]
fn no_depth_buffer_depth_write() {
    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::texture::Texture2d::empty_with_format(&display,
                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap, 128, 128).unwrap();
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();

    let parameters = glium::DrawParameters {
        depth: glium::Depth {
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    match framebuffer.draw(&vertex_buffer, &index_buffer, &program,
                           &glium::uniforms::EmptyUniforms, &parameters)
    {
        Err(glium::DrawError::NoDepthBuffer) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error(None);
}

#[test]
fn simple_dimensions() {
    let display = support::build_display();

    let texture = glium::Texture2d::empty_with_format(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap,
                                              128, 128).unwrap();

    let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    display.assert_no_error(None);
}

#[test]
fn simple_render_to_texture() {
    let display = support::build_display();
    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::Texture2d::empty_with_format(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap,
                                              128, 128).unwrap();

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(read_back[0][0], (255, 0, 0, 255));
    assert_eq!(read_back[64][64], (255, 0, 0, 255));
    assert_eq!(read_back[127][127], (255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn depth_texture2d() {
    use std::iter;

    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    // the program returns a Z coordinate between 0 (left of screen) and 1 (right of screen)
    let program = program!(&display,
        110 => {
            vertex: "
                #version 110

                attribute vec2 position;

                void main() {
                    gl_Position = vec4(position, position.x, 1.0);
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

                attribute lowp vec2 position;

                void main() {
                    gl_Position = vec4(position, position.x, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                }
            ",
        }).unwrap();

    // empty color attachment to put the data
    let color = glium::Texture2d::empty_with_format(&display,
                                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap,
                                            128, 128).unwrap();

    // depth texture with a value of 0.5 everywhere
    let depth_data = iter::repeat(iter::repeat(0.5f32).take(128).collect::<Vec<_>>())
                                  .take(128).collect::<Vec<_>>();
    let depth = match glium::texture::DepthTexture2d::new(&display, depth_data) {
        Err(_) => return,
        Ok(t) => t
    };

    // drawing with the `IfLess` depth test
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display,
                                                                                   &color, &depth).unwrap();
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            .. Default::default()
        },
        .. Default::default()
    };

    framebuffer.clear_color(0.0, 0.0, 0.0, 1.0);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    // reading back the color
    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = color.read();

    assert_eq!(read_back[0][0], (255, 255, 255, 255));
    assert_eq!(read_back[127][127], (0, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn multioutput() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let program = match glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 330

            out vec4 color1;
            out vec4 color2;

            void main() {
                color1 = vec4(1.0, 1.0, 1.0, 1.0);
                color2 = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
    {
        Err(glium::CompilationError(..)) => return,
        Ok(p) => p,
        e => e.unwrap()
    };

    // building two empty color attachments
    let color1 = glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::AutoGeneratedMipmaps,
                                               128, 128).unwrap();
    color1.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    let color2 = glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::AutoGeneratedMipmaps,
                                               128, 128).unwrap();
    color2.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    // building the framebuffer
    let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::new(&display,
                               [("color1", &color1), ("color2", &color2)].iter().cloned()).unwrap();

    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                     &Default::default()).unwrap();

    // checking color1
    let read_back1: Vec<Vec<(u8, u8, u8, u8)>> = color1.read();
    for row in read_back1.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 255, 255, 255));
        }
    }

    // checking color2
    let read_back2: Vec<Vec<(u8, u8, u8, u8)>> = color2.read();
    for row in read_back2.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }


    display.assert_no_error(None);
}

#[test]
fn array_level() {
    let display = support::build_display();

    let texture = match glium::texture::Texture2dArray::empty(&display, 128, 128, 4) {
        Ok(t) => t,
        Err(_) => return
    };

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display,
                                                          texture.main_level().layer(2).unwrap()).unwrap();
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                     &Default::default()).unwrap();

    // TODO: read the texture to see if it succeeded

    display.assert_no_error(None);
}

#[test]
fn cubemap_layer() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::env::var("TRAVIS").is_ok() {
        return;
    }

    let display = support::build_display();

    let texture = match glium::texture::Cubemap::empty(&display, 128) {
        Ok(t) => t,
        Err(_) => return
    };

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display,
                    texture.main_level().image(glium::texture::CubeLayer::PositiveY)).unwrap();
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                     &Default::default()).unwrap();

    // TODO: read the texture to see if it succeeded

    display.assert_no_error(None);
}

#[test]
#[should_panic]
fn multi_color_attachments_maximum() {
    let display = support::build_display();

    let color_textures = (0 .. 32)
        .map(|_| {
            glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::NoMipmap,
                                               128, 128).unwrap()
        })
        .collect::<Vec<_>>();

    let colors = (0 .. color_textures.len()).map(|i| {("attachment", &color_textures[i])} );
    glium::framebuffer::MultiOutputFrameBuffer::new(&display, colors).unwrap();
}

#[test]
#[should_panic]
fn empty_framebuffer_wrong_layers() {
    use glium::framebuffer::EmptyFrameBuffer;

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        panic!();
    }

    let _fb = EmptyFrameBuffer::new(&display, 256, 256, Some(0), None, true);
}

#[test]
#[should_panic]
fn empty_framebuffer_wrong_samples() {
    use glium::framebuffer::EmptyFrameBuffer;

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        panic!();
    }

    let _fb = EmptyFrameBuffer::new(&display, 256, 256, None, Some(0), true);
}

#[test]
fn empty_framebuffer_width_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 4294967295, 256, None, None, true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_height_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 256, 4294967295, None, None, true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_layers_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_layered_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 256, 256, Some(4294967295), None, true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_samples_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 256, 256, None, Some(4294967295), true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_simple_draw() {
    use glium::framebuffer::{EmptyFrameBuffer};

    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let mut fb = EmptyFrameBuffer::new(&display, 256, 256, None, None, true).unwrap();
    fb.clear_color(0.0, 0.0, 0.0, 0.0);
    fb.draw(&vertex_buffer, &index_buffer, &program,
            &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    display.assert_no_error(None);
}
