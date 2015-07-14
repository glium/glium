#[macro_use]
extern crate glium;

use glium::Surface;
use glium::Texture;

mod support;

#[test]
fn empty_texture1d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture1d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8, 128);

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}

#[test]
fn get_format_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8, 128, 128);

    display.assert_no_error(None);

    let format = match texture.get_internal_format_if_supported() {
        None => return,
        Some(f) => f
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

    let texture = match glium::texture::DepthTexture1d::new_if_supported(&display, vec![0.0, 0.0, 0.0, 0.0f32]) {
        None => return,
        Some(t) => t
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
    ]);

    assert_eq!(texture.get_width(), 2);
    assert_eq!(texture.get_height(), Some(3));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn empty_texture2d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8,
                                                       128, 128);

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}

#[test]
fn depth_texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::DepthTexture2d::new_if_supported(&display, vec![
        vec![0.0, 0.0, 0.0, 0.0f32],
        vec![0.0, 0.0, 0.0, 0.0f32],
        vec![0.0, 0.0, 0.0, 0.0f32],
    ]);

    let texture = match texture {
        None => return,
        Some(t) => t
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

    let texture = glium::texture::DepthTexture2d::new_empty(&display,
                                                            glium::texture::DepthFormat::F32,
                                                            128, 128);

    display.assert_no_error(None);
    drop(texture);
    display.assert_no_error(None);
}

#[test]
fn compressed_texture_2d_creation() {
    let display = support::build_display();

    let texture = glium::texture::CompressedTexture2d::new(&display, vec![
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0)],
        vec![(0, 0, 0, 0), (0, 0, 0, 0u8)],
    ]);

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

            let texture = glium::texture::$tex_ty::empty(&display, $($dims),+);

            assert_eq!(texture.get_width(), $w);
            assert_eq!(texture.get_height(), $h);
            assert_eq!(texture.get_depth(), $d);
            assert_eq!(texture.get_array_size(), $s);

            assert_eq!(texture.get_mipmap_levels(), 1);

            texture.get_internal_format_if_supported();

            display.assert_no_error(None);
            drop(texture);
            display.assert_no_error(None);
        }
    );

    ($test_name:ident, maybe $tex_ty:ident, [$($dims:expr),+],
     $w:expr, $h:expr, $d:expr, $s:expr) =>
    (
        #[test]
        fn $test_name() {
            let display = support::build_display();

            let texture = match glium::texture::$tex_ty::empty_if_supported(&display, $($dims),+) {
                None => return,
                Some(t) => t
            };

            assert_eq!(texture.get_width(), $w);
            assert_eq!(texture.get_height(), $h);
            assert_eq!(texture.get_depth(), $d);
            assert_eq!(texture.get_array_size(), $s);

            assert_eq!(texture.get_mipmap_levels(), 1);
            
            texture.get_internal_format_if_supported();

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
empty_texture_test!(empty_depthstenciltexture1d, maybe DepthStencilTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_depthstenciltexture1darray, maybe DepthStencilTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_depthstenciltexture2d, maybe DepthStencilTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_depthstenciltexture2darray, maybe DepthStencilTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
// TODO: non-working
//empty_texture_test!(empty_depthstenciltexture3d, DepthStencilTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_depthtexture1d, maybe DepthTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_depthtexture1darray, maybe DepthTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_depthtexture2d, maybe DepthTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_depthtexture2darray, maybe DepthTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
// TODO: non-working
//empty_texture_test!(empty_depthtexture3d, DepthTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_integraltexture1d, maybe IntegralTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_integraltexture1darray, maybe IntegralTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_integraltexture2d, maybe IntegralTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_integraltexture2darray, maybe IntegralTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_integraltexture3d, maybe IntegralTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_srgbtexture1d, maybe Texture1d, [64], 64, None, None, None);
empty_texture_test!(empty_srgbtexture1darray, maybe Texture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_srgbtexture2d, Texture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_srgbtexture2darray, maybe Texture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_srgbtexture3d, maybe Texture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_stenciltexture1d, maybe StencilTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_stenciltexture1darray, maybe StencilTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_stenciltexture2d, maybe StencilTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_stenciltexture2darray, maybe StencilTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_stenciltexture3d, maybe StencilTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_texture1d, maybe Texture1d, [64], 64, None, None, None);
empty_texture_test!(empty_texture1darray, maybe Texture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_texture2d, Texture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_texture2darray, maybe Texture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_texture3d, maybe Texture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_unsignedtexture1d, maybe UnsignedTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_unsignedtexture1darray, maybe UnsignedTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_unsignedtexture2d, maybe UnsignedTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_unsignedtexture2darray, maybe UnsignedTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_unsignedtexture3d, maybe UnsignedTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);

#[test]
fn zero_sized_texture_1d_creation() {
    let display = support::build_display();

    let texture = match glium::texture::Texture1d::new_if_supported(&display, Vec::<(u8, u8, u8, u8)>::new()) {
        None => return,
        Some(t) => t
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

    let texture = glium::texture::Texture2d::new(&display, Vec::<Vec<(u8, u8, u8, u8)>>::new());

    assert_eq!(texture.get_width(), 0);
    assert_eq!(texture.get_height(), Some(0));
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error(None);
}

#[test]
fn zero_sized_texture_3d_creation() {
    let display = support::build_display();

    let texture = match glium::texture::Texture3d::new_if_supported(&display, Vec::<Vec<Vec<(u8, u8, u8, u8)>>>::new()) {
        None => return,
        Some(t) => t
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
    ]);

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
