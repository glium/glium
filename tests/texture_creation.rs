extern crate glutin;
#[macro_use]
extern crate glium;

use glium::{Texture, Surface};

mod support;

#[test]
fn texture_1d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::Texture1d::new(&display, vec![
        (0, 0, 0, 0),
        (0, 0, 0, 0),
        (0, 0, 0, 0u8),
    ]);

    assert_eq!(texture.get_width(), 3);
    assert_eq!(texture.get_height(), None);
    assert_eq!(texture.get_depth(), None);
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
}

#[test]
fn empty_texture1d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture1d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8, 128);

    display.assert_no_error();
    drop(texture);
    display.assert_no_error();
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

    display.assert_no_error();
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

    display.assert_no_error();
}

#[test]
fn empty_texture2d_u8u8u8u8() {
    let display = support::build_display();

    let texture = glium::texture::Texture2d::new_empty(&display,
                                                       glium::texture::UncompressedFloatFormat::
                                                           U8U8U8U8,
                                                       128, 128);

    display.assert_no_error();
    drop(texture);
    display.assert_no_error();
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

    display.assert_no_error();
}

#[test]
#[ignore]   // `thread 'empty_depth_texture2d_f32' panicked at 'assertion failed: version >= &GlVersion(3, 0)'`
fn empty_depth_texture2d_f32() {
    let display = support::build_display();

    let texture = glium::texture::DepthTexture2d::new_empty(&display,
                                                            glium::texture::DepthFormat::F32,
                                                            128, 128);

    display.assert_no_error();
    drop(texture);
    display.assert_no_error();
}

#[test]
fn texture_3d_creation() {    
    let display = support::build_display();

    let texture = glium::texture::Texture3d::new(&display, vec![
        vec![
            vec![(0, 0, 0, 0)],
            vec![(0, 0, 0, 0)],
        ],
        vec![
            vec![(0, 0, 0, 0)],
            vec![(0, 0, 0, 0)],
        ],
        vec![
            vec![(0, 0, 0, 0)],
            vec![(0, 0, 0, 0u8)],
        ],
    ]);

    assert_eq!(texture.get_width(), 1);
    assert_eq!(texture.get_height(), Some(2));
    assert_eq!(texture.get_depth(), Some(3));
    assert_eq!(texture.get_array_size(), None);

    display.assert_no_error();
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

    display.assert_no_error();
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

            display.assert_no_error();
            drop(texture);
            display.assert_no_error();
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

            display.assert_no_error();
            drop(texture);
            display.assert_no_error();
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
empty_texture_test!(empty_integraltexture1darray,maybe  IntegralTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_integraltexture2d, maybe IntegralTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_integraltexture2darray, maybe IntegralTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_integraltexture3d, maybe IntegralTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_stenciltexture1d, maybe StencilTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_stenciltexture1darray, maybe StencilTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_stenciltexture2d, maybe StencilTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_stenciltexture2darray, maybe StencilTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_stenciltexture3d, maybe StencilTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_texture1d, Texture1d, [64], 64, None, None, None);
empty_texture_test!(empty_texture1darray, Texture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_texture2d, Texture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_texture2darray, Texture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_texture3d, Texture3d, [64, 32, 16], 64, Some(32), Some(16), None);
empty_texture_test!(empty_unsignedtexture1d, maybe UnsignedTexture1d, [64], 64, None, None, None);
empty_texture_test!(empty_unsignedtexture1darray, maybe UnsignedTexture1dArray, [64, 32], 64, None, None, Some(32));
empty_texture_test!(empty_unsignedtexture2d, maybe UnsignedTexture2d, [64, 32], 64, Some(32), None, None);
empty_texture_test!(empty_unsignedtexture2darray, maybe UnsignedTexture2dArray, [64, 32, 16], 64, Some(32), None, Some(16));
empty_texture_test!(empty_unsignedtexture3d, maybe UnsignedTexture3d, [64, 32, 16], 64, Some(32), Some(16), None);
