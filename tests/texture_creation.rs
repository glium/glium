#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn empty_texture1d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture1d::empty_with_format(&display,
                                                       glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                                       glium::texture::MipmapsOption::NoMipmap, 128);

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}

#[test]
fn get_format_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::empty_with_format(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                       U8U8U8U8,
                                                       glium::texture::MipmapsOption::NoMipmap,
                                                       128, 128).unwrap();

    display.assert_no_error(None);

    let format = match texture.get_internal_format() {
        Err(_) => return,
        Ok(f) => f
    };

    match format {
        glium::texture::InternalFormat::FourComponents { ty1, bits1, ty2, bits2, ty3, bits3, ty4, bits4 } => {
            assert_eq!(ty1, glium::texture::InternalFormatType::UnsignedNormalized);
            assert_eq!(ty2, glium::texture::InternalFormatType::UnsignedNormalized);
            assert_eq!(ty3, glium::texture::InternalFormatType::UnsignedNormalized);
            assert_eq!(ty4, glium::texture::InternalFormatType::UnsignedNormalized);

            assert!(bits1 >= 8);
            assert!(bits2 >= 8);
            assert!(bits3 >= 8);
            assert!(bits4 >= 8);
        },
        _ => panic!()
    }
}

#[test]
fn depth_texture_1d_creation() {
    let display = support::build_display();

    let texture = match glium::texture::DepthTexture1d::new(&display, vec![0.0, 0.0, 0.0, 0.0f32]) {
        Err(_) => return,       // TODO: not supported error
        Ok(t) => t
    };

    assert_eq!(texture.get_width(), 4);
    assert_eq!(texture.get_height(), None);
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0u8)],
    ]).unwrap();

    assert_eq!(texture.get_width(), 2);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn texture_2d_as_uniform_value_lifetime() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0u8)],
    ]).unwrap();

    // A function that takes texture reference should be able to return UniformValue (with lifetime
    // inherited from the reference).
    fn get_uniforms(texture: &glium::texture::Texture2d) -> glium::uniforms::UniformValue {
        use glium::uniforms::AsUniformValue;
        texture.as_uniform_value()
    }

    let uniforms = get_uniforms(&texture);
    assert!(matches!(uniforms, glium::uniforms::UniformValue::Texture2d(..)));

    display.assert_no_error(None);
}

#[test]
fn empty_texture2d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::empty_with_format(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8,
                                                        glium::texture::MipmapsOption::NoMipmap,
                                                       128, 128);

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}

#[test]
fn depth_texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::DepthTexture2d::new(&display, vec![
        vec![0.0, 0.0, 0.0, 0.0f32],
        vec![0.0, 0.0, 0.0, 0.0f32],
        vec![0.0, 0.0, 0.0, 0.0f32],
    ]);

    let texture = match texture {
        Err(_) => return,       // TODO: not supported error
        Ok(t) => t
    };

    assert_eq!(texture.get_width(), 4);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
#[ignore]   // `thread 'empty_depth_texture2d_f32' panicked at 'assertion failed: version >= &GlVersion(3, 0)'`
fn empty_depth_texture2d_f32() {
    let display = support::build_display();

    let _texture = glium::texture::DepthTexture2d::empty(&display, 128, 128);

    display.assert_no_error(None);
    drop(_texture);
    display.assert_no_error(None);
}

#[test]
fn compressed_texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::CompressedTexture2d::new(&display, vec![
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0u8)],
    ]).unwrap();

    assert_eq!(texture.get_width(), 2);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

macro_rules! empty_texture_test {
    ($test_name:ident, $tex_ty:ident, [$($dims:expr),+],
     $w:expr, $h:expr, $d:expr, $s:expr) =>
    (
        #[test]
        fn $test_name() {
            let display = support::build_display();

            let texture = match glium::texture::$tex_ty::empty(&display, $($dims),+) {
                Err(_) => return,       // TODO: make sure it's `NotSupported`
                Ok(tex) => tex
            };

            assert_eq!(texture.get_width(), $w);
            assert_eq!(texture.get_height(), $h);
            assert_eq!(texture.get_depth(), $d);
            assert_eq!(texture.get_array_size(), $s);

            assert_eq!(texture.get_mipmap_levels(), 1);

            let _ = texture.get_internal_format();

            display.assert_no_error(None);
            drop(texture);
            display.assert_no_error(None);
        }
    );
}

// TODO: compressed textures don't have "empty" yet
/*empty_texture_test!(empty_compressedtexture1d, CompressedTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_compressedtexture1darray, CompressedTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_compressedtexture2d, CompressedTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_compressedtexture2darray, CompressedTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_compressedtexture3d, CompressedTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);*/
empty_texture_test!(empty_depthstenciltexture1d, DepthStencilTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_depthstenciltexture1darray, DepthStencilTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_depthstenciltexture2d, DepthStencilTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_depthstenciltexture2darray, DepthStencilTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
// TODO: non-working
//empty_texture_test!(empty_depthstenciltexture3d, DepthStencilTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_depthtexture1d, DepthTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_depthtexture1darray, DepthTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_depthtexture2d, DepthTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_depthtexture2darray, DepthTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
// TODO: non-working
//empty_texture_test!(empty_depthtexture3d, DepthTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_integraltexture1d, IntegralTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_integraltexture1darray, IntegralTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_integraltexture2d, IntegralTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_integraltexture2darray, IntegralTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_integraltexture3d, IntegralTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_srgbtexture1d, Texture1d, [64], 64, None, None, None);
empty_texture_test!(empty_srgbtexture1darray, Texture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_srgbtexture2d, Texture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_srgbtexture2darray, Texture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_srgbtexture3d, Texture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_stenciltexture1d, StencilTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_stenciltexture1darray, StencilTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_stenciltexture2d, StencilTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_stenciltexture2darray, StencilTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
//empty_texture_test!(empty_stenciltexture3d, StencilTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_texture1d, Texture1d, [64], 64, None, None, None);
empty_texture_test!(empty_texture1darray, Texture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_texture2d, Texture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_texture2darray, Texture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_texture3d, Texture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_unsignedtexture1d, UnsignedTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_unsignedtexture1darray, UnsignedTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_unsignedtexture2d, UnsignedTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_unsignedtexture2darray, UnsignedTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_unsignedtexture3d, UnsignedTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);

#[test]
fn zero_sized_texture_1d_creation() {
    let display = support::build_display();

    let texture = match glium::texture::Texture1d::new(&display, Vec::<(u8, u8, u8, u8)>::new()) {
        Err(_) => return,       // TODO: make sure it's `NotSupported`
        Ok(t) => t
    };

    assert_eq!(texture.get_width(), 0);
    assert_eq!(texture.get_height(), None);
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn zero_sized_texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new(&display, Vec::<Vec<(u8, u8, u8, u8)>>::new()).unwrap();

    assert_eq!(texture.get_width(), 0);
    assert_eq!(texture.get_height(), Some(0));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn zero_sized_texture_3d_creation() {
    let display = support::build_display();

    let texture = match glium::texture::Texture3d::new(&display, Vec::<Vec<Vec<(u8, u8, u8, u8)>>>::new()) {
        Err(_) => return,       // TODO: make sure it's `NotSupported`
        Ok(t) => t
    };

    assert_eq!(texture.get_width(), 0);
    assert_eq!(texture.get_height(), Some(0));
    assert_eq!(texture.get_depth(), Some(0));
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn bindless_texture_residency_context_rebuild() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let texture = glium::texture::Texture2d::new(&display, vec![
        vec![(255, 0, 0, 255), (255, 0, 0, 255)],
        vec![(255, 0, 0, 255), (255, 0, 0, 255u8)],
    ]).unwrap();

    let texture = match texture.resident() {
        Ok(t) => t,
        Err(_) => return
    };

    // here is the trick: we rebuild the display, meaning that texture residency has to be updated
    // by glium
    support::rebuild_display(&display);

    // if bindless textures are supported, we can call .unwrap() and expect that everything
    // else is supported here as well

    let program = glium::Program::from_source(&display,
        "
            #version 100

            attribute lowp vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 400
            #extension GL_ARB_bindless_texture : require

            uniform Samplers {
                sampler2D tex;
            };

            out vec4 f_color;

            void main() {
                f_color = texture(tex, vec2(0.0, 0.0));
            }
        ",
        None).unwrap();

    let buffer = glium::uniforms::UniformBuffer::new(&display,
                                            glium::texture::TextureHandle::new(&texture, &Default::default())).unwrap();

    let output = support::build_renderable_texture(&display);
    output.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    output.as_surface().draw(&vb, &ib, &program, &uniform!{ Samplers: &buffer },
                             &Default::default()).unwrap();

    let data: Vec<Vec<(u8, u8, u8, u8)>> = output.read();
    for row in data.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }

    display.assert_no_error(None);
}

#[test]
fn upload_from_pixel_buffer() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::empty(&display, 2, 2).unwrap();

    let buffer = glium::texture::pixel_buffer::PixelBuffer::new_empty(&display, 4);
    buffer.write(&[(0u8, 255u8, 0u8, 255u8), (255, 0, 255, 0), (255, 255, 0, 255),
                   (0, 0, 255, 255)]);

    texture.main_level().raw_upload_from_pixel_buffer(buffer.as_slice(), 0 .. 2, 0 .. 2, 0 .. 1);


    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (0, 255, 0, 255));
    assert_eq!(data[0][1], (255, 0, 255, 0));
    assert_eq!(data[1][0], (255, 255, 0, 255));
    assert_eq!(data[1][1], (0, 0, 255, 255));

    display.assert_no_error(None);
}

#[test]
fn upload_from_pixel_buffer_inverted() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::empty(&display, 2, 2).unwrap();

    let buffer = glium::texture::pixel_buffer::PixelBuffer::new_empty(&display, 4);
    buffer.write(&[(0u8, 255u8, 0u8, 255u8), (255, 0, 255, 0), (255, 255, 0, 255),
                   (0, 0, 255, 255)]);

    texture.main_level().raw_upload_from_pixel_buffer_inverted(buffer.as_slice(), 0 .. 2, 0 .. 2, 0 .. 1);


    let data: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();
    assert_eq!(data[0][0], (0, 255, 0, 255));
    assert_eq!(data[0][1], (255, 0, 255, 0));
    assert_eq!(data[1][0], (0, 255, 255, 255));
    assert_eq!(data[1][1], (255, 0, 0, 255));

    display.assert_no_error(None);
}
