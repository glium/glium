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
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);

    let parameters = glium::DrawParameters {
        depth_test: glium::DepthTest::IfLess,
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
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);

    let parameters = glium::DrawParameters {
        depth_write: true,
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

    let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
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

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture);
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
                                                                                   &color, &depth);
    let params = glium::DrawParameters {
        depth_test: glium::DepthTest::IfLess,
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
        Err(glium::CompilationError(_)) => return,
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
                                             &[("color1", &color1), ("color2", &color2)]);

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

    let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display,
                                                          texture.main_level().layer(2).unwrap());
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    display.assert_no_error(None);
}
