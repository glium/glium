use gl;
use GlObject;

use backend::Facade;
use version::Version;
use context::Context;
use context::CommandContext;
use CapabilitiesSource;
use ContextExt;
use TextureExt;
use TextureMipmapExt;
use version::Api;
use Rect;

use image_format::{self, TextureFormatRequest, ClientFormatAny};
use texture::Texture2dDataSink;
use texture::TextureKind;
use texture::{MipmapsOption, TextureFormat, TextureCreationError, CubeLayer};
use texture::{get_format, InternalFormat, GetFormatError};
use texture::pixel::PixelValue;
use texture::pixel_buffer::PixelBuffer;

use fbo::ClearBufferData;

use buffer::BufferSlice;
use buffer::BufferAny;
use BufferExt;
use BufferSliceExt;

use std::cmp;
use std::fmt;
use std::mem;
use std::ptr;
use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;
use std::ops::Range;
use std::ffi::c_void;

use ops;
use fbo;

/// Type of a texture.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]      // TODO: document and remove
pub enum Dimensions {
    Texture1d { width: u32 },
    Texture1dArray { width: u32, array_size: u32 },
    Texture2d { width: u32, height: u32 },
    Texture2dArray { width: u32, height: u32, array_size: u32 },
    Texture2dMultisample { width: u32, height: u32, samples: u32 },
    Texture2dMultisampleArray { width: u32, height: u32, array_size: u32, samples: u32 },
    Texture3d { width: u32, height: u32, depth: u32 },
    Cubemap { dimension: u32 },
    CubemapArray { dimension: u32, array_size: u32 },
}

/// A texture whose type isn't fixed at compile-time.
pub struct TextureAny {
    context: Rc<Context>,
    id: gl::types::GLuint,
    requested_format: TextureFormatRequest,

    /// Cache for the actual format of the texture. The outer Option is None if the format hasn't
    /// been checked yet. The inner Result is Err if the format has been checked but is unknown.
    actual_format: Cell<Option<Result<InternalFormat, GetFormatError>>>,

    /// Type and dimensions of the texture.
    ty: Dimensions,

    /// Number of mipmap levels (`1` means just the main texture, `0` is not valid)
    levels: u32,
    /// Is automatic mipmap generation allowed for this texture?
    generate_mipmaps: bool,

    /// Is this texture owned by us? If not, we won't clean it up on drop.
    owned: bool
}

fn extract_dimensions(ty: Dimensions)
                      -> (u32, Option<u32>, Option<u32>, Option<u32>, Option<u32>)
{
    match ty {
        Dimensions::Texture1d { width } => (width, None, None, None, None),
        Dimensions::Texture1dArray { width, array_size } => (width, None, None, Some(array_size), None),
        Dimensions::Texture2d { width, height } => (width, Some(height), None, None, None),
        Dimensions::Texture2dArray { width, height, array_size } => (width, Some(height), None, Some(array_size), None),
        Dimensions::Texture2dMultisample { width, height, samples } => (width, Some(height), None, None, Some(samples)),
        Dimensions::Texture2dMultisampleArray { width, height, array_size, samples } => (width, Some(height), None, Some(array_size), Some(samples)),
        Dimensions::Texture3d { width, height, depth } => (width, Some(height), Some(depth), None, None),
        Dimensions::Cubemap { dimension } => (dimension, Some(dimension), None, None, None),
        Dimensions::CubemapArray { dimension, array_size } => (dimension, Some(dimension), None, Some(array_size * 6), None),
    }
}

#[inline]
fn get_bind_point(ty: Dimensions) -> gl::types::GLenum {
    match ty {
        Dimensions::Texture1d { .. } => gl::TEXTURE_1D,
        Dimensions::Texture1dArray { .. } => gl::TEXTURE_1D_ARRAY,
        Dimensions::Texture2d { .. } => gl::TEXTURE_2D,
        Dimensions::Texture2dArray { .. } => gl::TEXTURE_2D_ARRAY,
        Dimensions::Texture2dMultisample { .. } => gl::TEXTURE_2D_MULTISAMPLE,
        Dimensions::Texture2dMultisampleArray { .. } => gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
        Dimensions::Texture3d { .. } => gl::TEXTURE_3D,
        Dimensions::Cubemap { .. } => gl::TEXTURE_CUBE_MAP,
        Dimensions::CubemapArray { .. } => gl::TEXTURE_CUBE_MAP_ARRAY,
    }
}

unsafe fn generate_mipmaps(ctxt: &CommandContext,
                           bind_point: gl::types::GLenum) {
    if ctxt.version >= &Version(Api::Gl, 3, 0) ||
       ctxt.version >= &Version(Api::GlEs, 2, 0)
    {
        ctxt.gl.GenerateMipmap(bind_point);
    } else if ctxt.extensions.gl_ext_framebuffer_object {
        ctxt.gl.GenerateMipmapEXT(bind_point);
    } else {
        unreachable!();
    }
}

/// Builds a new texture.
///
/// # Panic
///
/// Panics if the size of the data doesn't match the texture dimensions.
pub fn new_texture<'a, F: ?Sized, P>(facade: &F, format: TextureFormatRequest,
                             data: Option<(ClientFormatAny, Cow<'a, [P]>)>,
                             mipmaps: MipmapsOption, ty: Dimensions)
                             -> Result<TextureAny, TextureCreationError>
                             where P: Send + Clone + 'a, F: Facade
{
    // getting the width, height, depth, array_size, samples from the type
    let (width, height, depth, array_size, samples) = extract_dimensions(ty);
    let (is_client_compressed, data_bufsize) = match data {
        Some((client_format, _)) => {
            (client_format.is_compressed(),
             client_format.get_buffer_size(width, height, depth, array_size))
        },
        None => (false, 0),
    };

    if let Some((_, ref data)) = data {
        if data.len() * mem::size_of::<P>() != data_bufsize
        {
            panic!("Texture data size mismatch");
        }
    }

    // getting the `GLenum` corresponding to this texture type
    let bind_point = get_bind_point(ty);
    if bind_point == gl::TEXTURE_CUBE_MAP || bind_point == gl::TEXTURE_CUBE_MAP_ARRAY {
        assert!(data.is_none());        // TODO: not supported
    }

    // checking non-power-of-two
    if facade.get_context().get_version() < &Version(Api::Gl, 2, 0) &&
        !facade.get_context().get_extensions().gl_arb_texture_non_power_of_two
    {
        if !width.is_power_of_two() || !height.unwrap_or(2).is_power_of_two() ||
            !depth.unwrap_or(2).is_power_of_two() || !array_size.unwrap_or(2).is_power_of_two()
        {
            return Err(TextureCreationError::DimensionsNotSupported);
        }
    }

    let should_generate_mipmaps = mipmaps.should_generate();
    let texture_levels = mipmaps.num_levels(width, height, depth) as gl::types::GLsizei;

    let teximg_internal_format = image_format::format_request_to_glenum(facade.get_context(), format, image_format::RequestType::TexImage(data.as_ref().map(|&(c, _)| c)))?;
    let storage_internal_format = image_format::format_request_to_glenum(facade.get_context(), format, image_format::RequestType::TexStorage).ok();

    let (client_format, client_type) = match (&data, format) {
        (&Some((client_format, _)), f) => image_format::client_format_to_glenum(facade.get_context(), client_format, f, false)?,
        (&None, TextureFormatRequest::AnyDepth) => (gl::DEPTH_COMPONENT, gl::FLOAT),
        (&None, TextureFormatRequest::Specific(TextureFormat::DepthFormat(_))) => (gl::DEPTH_COMPONENT, gl::FLOAT),
        (&None, TextureFormatRequest::AnyDepthStencil) => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
        (&None, TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_))) => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
        (&None, _) => (gl::RGBA, gl::UNSIGNED_BYTE),
    };

    let (filtering, mipmap_filtering) = match format {
        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) => (gl::NEAREST, gl::NEAREST_MIPMAP_NEAREST),
        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) => (gl::NEAREST, gl::NEAREST_MIPMAP_NEAREST),
        TextureFormatRequest::Specific(TextureFormat::StencilFormat(_)) => (gl::NEAREST, gl::NEAREST_MIPMAP_NEAREST),
        TextureFormatRequest::AnyIntegral => (gl::NEAREST, gl::NEAREST_MIPMAP_NEAREST),
        TextureFormatRequest::AnyUnsigned => (gl::NEAREST, gl::NEAREST_MIPMAP_NEAREST),
        TextureFormatRequest::AnyStencil => (gl::NEAREST, gl::NEAREST_MIPMAP_NEAREST),
        _ => (gl::LINEAR, gl::LINEAR_MIPMAP_LINEAR),
    };

    let is_multisampled = match ty {
        Dimensions::Texture2dMultisample {..}
        | Dimensions::Texture2dMultisampleArray {..} => true,
        _ => false,
    };

    let mut ctxt = facade.get_context().make_current();

    let id = unsafe {
        let has_mipmaps = texture_levels > 1;
        let data = data;
        let data_raw = if let Some((_, ref data)) = data {
            data.as_ptr() as *const c_void
        } else {
            ptr::null()
        };

        if ctxt.state.pixel_store_unpack_alignment != 1 {
            ctxt.state.pixel_store_unpack_alignment = 1;
            ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        BufferAny::unbind_pixel_unpack(&mut ctxt);

        let id: gl::types::GLuint = 0;
        ctxt.gl.GenTextures(1, mem::transmute(&id));

        {
            ctxt.gl.BindTexture(bind_point, id);
            let act = ctxt.state.active_texture as usize;
            ctxt.state.texture_units[act].texture = id;
        }

        if !is_multisampled {
            ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_MAG_FILTER, filtering as i32);
        }

        match ty {
            Dimensions::Texture1d { .. } => (),
            Dimensions::Texture2dMultisample { .. } => (),
            Dimensions::Texture2dMultisampleArray { .. } => (),
            _ => {
                ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            },
        };

        match ty {
            Dimensions::Texture1d { .. } => (),
            Dimensions::Texture2d { .. } => (),
            Dimensions::Texture2dMultisample { .. } => (),
            _ => {
                ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
            },
        };

        if has_mipmaps {
            ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_MIN_FILTER,
                                  mipmap_filtering as i32);
        } else if !is_multisampled {
            ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_MIN_FILTER,
                                  filtering as i32);
        }

        if !has_mipmaps && (ctxt.version >= &Version(Api::Gl, 1, 2) ||
                            ctxt.version >= &Version(Api::GlEs, 3, 0))
        {
            ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_BASE_LEVEL, 0);
            ctxt.gl.TexParameteri(bind_point, gl::TEXTURE_MAX_LEVEL, 0);
        }

        if bind_point == gl::TEXTURE_3D || bind_point == gl::TEXTURE_2D_ARRAY ||
           bind_point == gl::TEXTURE_CUBE_MAP_ARRAY
        {
            let mut data_raw = data_raw;

            let width = match width as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            let height = match height.unwrap() as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            let depth = match depth.or(array_size).unwrap() as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            if storage_internal_format.is_some() && (ctxt.version >= &Version(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                ctxt.gl.TexStorage3D(bind_point, texture_levels,
                                     storage_internal_format.unwrap() as gl::types::GLenum,
                                     width, height, depth);

                if !data_raw.is_null() {
                    if is_client_compressed {
                        ctxt.gl.CompressedTexSubImage3D(bind_point, 0, 0, 0, 0, width, height, depth,
                                                         teximg_internal_format as u32,
                                                         data_bufsize as i32, data_raw);
                    } else {
                        ctxt.gl.TexSubImage3D(bind_point, 0, 0, 0, 0, width, height, depth,
                                              client_format, client_type, data_raw);
                    }
                }

            } else {
                if is_client_compressed && !data_raw.is_null() {
                    ctxt.gl.CompressedTexImage3D(bind_point, 0, teximg_internal_format as u32,
                                       width, height, depth, 0, data_bufsize as i32, data_raw);
                } else {
                    ctxt.gl.TexImage3D(bind_point, 0, teximg_internal_format as i32, width,
                                       height, depth, 0, client_format as u32, client_type,
                                       data_raw);
                }
            }

        } else if bind_point == gl::TEXTURE_2D || bind_point == gl::TEXTURE_1D_ARRAY ||
                  bind_point == gl::TEXTURE_CUBE_MAP
        {
            let mut data_raw = data_raw;

            let width = match width as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            let height = match height.or(array_size).unwrap() as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            if storage_internal_format.is_some() && (ctxt.version >= &Version(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                ctxt.gl.TexStorage2D(bind_point, texture_levels,
                                     storage_internal_format.unwrap() as gl::types::GLenum,
                                     width, height);

                if !data_raw.is_null() {
                    if is_client_compressed {
                        ctxt.gl.CompressedTexSubImage2D(bind_point, 0, 0, 0, width, height,
                                                         teximg_internal_format as u32,
                                                         data_bufsize as i32, data_raw);
                    } else {
                        ctxt.gl.TexSubImage2D(bind_point, 0, 0, 0, width, height, client_format,
                                              client_type, data_raw);
                    }
                }

            } else {
                if is_client_compressed && !data_raw.is_null() {
                    ctxt.gl.CompressedTexImage2D(bind_point, 0, teximg_internal_format as u32,
                                       width, height, 0, data_bufsize as i32, data_raw);
                } else {
                    ctxt.gl.TexImage2D(bind_point, 0, teximg_internal_format as i32, width,
                                       height, 0, client_format as u32, client_type, data_raw);
                }
            }

        } else if bind_point == gl::TEXTURE_2D_MULTISAMPLE {
            assert!(data_raw.is_null());

            let width = match width as gl::types::GLsizei {
                0 => 1,
                a => a
            };

            let height = match height.unwrap() as gl::types::GLsizei {
                0 => 1,
                a => a
            };

            if storage_internal_format.is_some() && (ctxt.version >= &Version(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                ctxt.gl.TexStorage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE,
                                                samples.unwrap() as gl::types::GLsizei,
                                                storage_internal_format.unwrap() as gl::types::GLenum,
                                                width, height, gl::TRUE);

            } else if ctxt.version >= &Version(Api::Gl, 3, 2) || ctxt.extensions.gl_arb_texture_multisample {
                ctxt.gl.TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE,
                                              samples.unwrap() as gl::types::GLsizei,
                                              teximg_internal_format as gl::types::GLenum,
                                              width, height, gl::TRUE);

            } else {
                unreachable!();
            }

        } else if bind_point == gl::TEXTURE_2D_MULTISAMPLE_ARRAY {
            assert!(data_raw.is_null());

            let width = match width as gl::types::GLsizei {
                0 => 1,
                a => a
            };

            let height = match height.unwrap() as gl::types::GLsizei {
                0 => 1,
                a => a
            };

            if storage_internal_format.is_some() && (ctxt.version >= &Version(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                ctxt.gl.TexStorage3DMultisample(gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
                                                samples.unwrap() as gl::types::GLsizei,
                                                storage_internal_format.unwrap() as gl::types::GLenum,
                                                width, height, array_size.unwrap() as gl::types::GLsizei,
                                                gl::TRUE);

            } else if ctxt.version >= &Version(Api::Gl, 3, 2) || ctxt.extensions.gl_arb_texture_multisample {
                ctxt.gl.TexImage3DMultisample(gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
                                              samples.unwrap() as gl::types::GLsizei,
                                              teximg_internal_format as gl::types::GLenum,
                                              width, height, array_size.unwrap() as gl::types::GLsizei,
                                              gl::TRUE);

            } else {
                unreachable!();
            }

        } else if bind_point == gl::TEXTURE_1D {
            let mut data_raw = data_raw;

            let width = match width as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            if storage_internal_format.is_some() && (ctxt.version >= &Version(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                ctxt.gl.TexStorage1D(bind_point, texture_levels,
                                     storage_internal_format.unwrap() as gl::types::GLenum,
                                     width);

                if !data_raw.is_null() {
                    if is_client_compressed {
                        ctxt.gl.CompressedTexSubImage1D(bind_point, 0, 0, width,
                                                         teximg_internal_format as u32,
                                                         data_bufsize as i32, data_raw);
                    } else {
                        ctxt.gl.TexSubImage1D(bind_point, 0, 0, width, client_format,
                                              client_type, data_raw);
                    }
                }

            } else {
                if is_client_compressed && !data_raw.is_null() {
                    ctxt.gl.CompressedTexImage1D(bind_point, 0, teximg_internal_format as u32,
                                       width, 0, data_bufsize as i32, data_raw);
                } else {
                    ctxt.gl.TexImage1D(bind_point, 0, teximg_internal_format as i32, width,
                                       0, client_format as u32, client_type, data_raw);
                }
            }

        } else {
            unreachable!();
        }

        // only generate mipmaps for color textures
        if should_generate_mipmaps {
            generate_mipmaps(&ctxt, bind_point);
        }

        id
    };

    Ok(TextureAny {
        context: facade.get_context().clone(),
        id: id,
        requested_format: format,
        actual_format: Cell::new(None),
        ty: ty,
        levels: texture_levels as u32,
        generate_mipmaps: should_generate_mipmaps,
        owned: true
    })
}

/// Builds a new texture reference from an existing, externally created OpenGL texture.
/// If `owned` is true, this reference will take ownership of the texture and be responsible
/// for cleaning it up. Otherwise, the texture must be cleaned up externally, but only
/// after this reference's lifetime has ended.
pub unsafe fn from_id<F: Facade + ?Sized>(facade: &F,
                                 format: TextureFormatRequest,
                                 id: gl::types::GLuint,
                                 owned: bool,
                                 mipmaps: MipmapsOption,
                                 ty: Dimensions)
                                 -> TextureAny {
    let (width, height, depth, array_size, samples) = extract_dimensions(ty);
    let mipmap_levels = mipmaps.num_levels(width, height, depth);
    let should_generate_mipmaps = mipmaps.should_generate();
    if should_generate_mipmaps {
        let ctxt = facade.get_context().make_current();
        generate_mipmaps(&ctxt, get_bind_point(ty));
    }
    TextureAny {
        context: facade.get_context().clone(),
        id: id,
        requested_format: format,
        actual_format: Cell::new(None),
        ty: ty,
        levels: mipmap_levels,
        generate_mipmaps: should_generate_mipmaps,
        owned: owned
    }
}

impl TextureAny {
    /// Returns the width of the texture.
    #[inline]
    pub fn get_width(&self) -> u32 {
        match self.ty {
            Dimensions::Texture1d { width, .. } => width,
            Dimensions::Texture1dArray { width, .. } => width,
            Dimensions::Texture2d { width, .. } => width,
            Dimensions::Texture2dArray { width, .. } => width,
            Dimensions::Texture2dMultisample { width, .. } => width,
            Dimensions::Texture2dMultisampleArray { width, .. } => width,
            Dimensions::Texture3d { width, .. } => width,
            Dimensions::Cubemap { dimension, .. } => dimension,
            Dimensions::CubemapArray { dimension, .. } => dimension,
        }
    }

    /// Returns the height of the texture.
    #[inline]
    pub fn get_height(&self) -> Option<u32> {
        match self.ty {
            Dimensions::Texture1d { .. } => None,
            Dimensions::Texture1dArray { .. } => None,
            Dimensions::Texture2d { height, .. } => Some(height),
            Dimensions::Texture2dArray { height, .. } => Some(height),
            Dimensions::Texture2dMultisample { height, .. } => Some(height),
            Dimensions::Texture2dMultisampleArray { height, .. } => Some(height),
            Dimensions::Texture3d { height, .. } => Some(height),
            Dimensions::Cubemap { dimension, .. } => Some(dimension),
            Dimensions::CubemapArray { dimension, .. } => Some(dimension),
        }
    }

    /// Returns the depth of the texture.
    #[inline]
    pub fn get_depth(&self) -> Option<u32> {
        match self.ty {
            Dimensions::Texture3d { depth, .. } => Some(depth),
            _ => None
        }
    }

    /// Returns the initial requested format.
    #[inline]
    #[doc(hidden)]
    pub fn get_requested_format(&self) -> TextureFormatRequest {
        self.requested_format
    }

    /// Returns the kind of texture.
    #[inline]
    pub fn kind(&self) -> TextureKind {
        match self.requested_format {
            TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(_)) => TextureKind::Float,
            TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) => TextureKind::Integral,
            TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) => TextureKind::Unsigned,
            TextureFormatRequest::Specific(TextureFormat::Srgb(_)) => TextureKind::Float,
            TextureFormatRequest::Specific(TextureFormat::CompressedFormat(_)) => TextureKind::Float,
            TextureFormatRequest::Specific(TextureFormat::CompressedSrgbFormat(_)) => TextureKind::Float,
            TextureFormatRequest::Specific(TextureFormat::DepthFormat(_)) => TextureKind::Depth,
            TextureFormatRequest::Specific(TextureFormat::StencilFormat(_)) => TextureKind::Stencil,
            TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_)) => TextureKind::DepthStencil,
            TextureFormatRequest::AnyFloatingPoint => TextureKind::Float,
            TextureFormatRequest::AnyCompressed => TextureKind::Float,
            TextureFormatRequest::AnySrgb => TextureKind::Float,
            TextureFormatRequest::AnyCompressedSrgb => TextureKind::Float,
            TextureFormatRequest::AnyIntegral => TextureKind::Integral,
            TextureFormatRequest::AnyUnsigned => TextureKind::Unsigned,
            TextureFormatRequest::AnyDepth => TextureKind::Depth,
            TextureFormatRequest::AnyStencil => TextureKind::Stencil,
            TextureFormatRequest::AnyDepthStencil => TextureKind::DepthStencil,
        }
    }

    /// Returns the dimensions of the texture.
    #[inline]
    pub fn dimensions(&self) -> Dimensions {
        self.ty.clone()
    }

    /// Returns the array size of the texture.
    #[inline]
    pub fn get_array_size(&self) -> Option<u32> {
        match self.ty {
            Dimensions::Texture1d { .. } => None,
            Dimensions::Texture1dArray { array_size, .. } => Some(array_size),
            Dimensions::Texture2d { .. } => None,
            Dimensions::Texture2dArray { array_size, .. } => Some(array_size),
            Dimensions::Texture2dMultisample { .. } => None,
            Dimensions::Texture2dMultisampleArray { array_size, .. } => Some(array_size),
            Dimensions::Texture3d { .. } => None,
            Dimensions::Cubemap { .. } => None,
            Dimensions::CubemapArray { array_size, .. } => Some(array_size),
        }
    }

    /// Returns the number of samples of the texture if it is a multisampling texture.
    #[inline]
    pub fn get_samples(&self) -> Option<u32> {
        match self.ty {
            Dimensions::Texture2dMultisample { samples, .. } => Some(samples),
            Dimensions::Texture2dMultisampleArray { samples, .. } => Some(samples),
            _ => None
        }
    }

    /// Returns a structure that represents the first layer of the texture. All textures have a
    /// first layer.
    #[inline]
    pub fn first_layer(&self) -> TextureAnyLayer {
        self.layer(0).unwrap()
    }

    /// Returns a structure that represents a specific layer of the texture.
    ///
    /// Non-array textures have only one layer. The number of layers can be queried with
    /// `get_array_size`.
    ///
    /// Returns `None` if out of range.
    #[inline]
    pub fn layer(&self, layer: u32) -> Option<TextureAnyLayer> {
        if layer >= self.get_array_size().unwrap_or(1) {
            return None;
        }

        Some(TextureAnyLayer {
            texture: self,
            layer: layer,
        })
    }

    /// Returns the type of the texture (1D, 2D, 3D, etc.).
    #[inline]
    pub fn get_texture_type(&self) -> Dimensions {
        self.ty
    }

    /// Determines the internal format of this texture.
    #[inline]
    pub fn get_internal_format(&self) -> Result<InternalFormat, GetFormatError> {
        if let Some(format) = self.actual_format.get() {
            format

        } else {
            let mut ctxt = self.context.make_current();
            let format = get_format::get_format(&mut ctxt, self);
            self.actual_format.set(Some(format.clone()));
            format
        }
    }

    /// Determines the number of depth and stencil bits in the format of this texture.
    pub fn get_depth_stencil_bits(&self) -> (u16, u16) {
        unsafe {
            let ctxt = self.context.make_current();
            let mut depth_bits: gl::types::GLint = 0;
            let mut stencil_bits: gl::types::GLint = 0;
            // FIXME: GL version considerations
            ctxt.gl.GetTextureLevelParameteriv(self.id, 0, gl::TEXTURE_DEPTH_SIZE, &mut depth_bits);
            ctxt.gl.GetTextureLevelParameteriv(self.id, 0, gl::TEXTURE_STENCIL_SIZE, &mut stencil_bits);
            (depth_bits as u16, stencil_bits as u16)
        }
    }

    /// Returns the number of mipmap levels of the texture.
    #[inline]
    pub fn get_mipmap_levels(&self) -> u32 {
        self.levels
    }

    /// Returns a structure that represents the main mipmap level of the texture.
    #[inline]
    pub fn main_level(&self) -> TextureAnyMipmap {
        self.mipmap(0).unwrap()
    }

    /// Returns a structure that represents a specific mipmap of the texture.
    ///
    /// Returns `None` if out of range.
    #[inline]
    pub fn mipmap(&self, level: u32) -> Option<TextureAnyMipmap> {
        if level >= self.levels {
            return None;
        }

        let pow = 2u32.pow(level);
        Some(TextureAnyMipmap {
            texture: self,
            level: level,
            width: cmp::max(1, self.get_width() / pow),
            height: self.get_height().map(|height| cmp::max(1, height / pow)),
            depth: self.get_depth().map(|depth| cmp::max(1, depth / pow)),
        })
    }

    /// Binds this texture and generates mipmaps.
    #[inline]
    pub unsafe fn generate_mipmaps(&self) {
        let mut ctxt = self.context.make_current();
        self.bind_to_current(&mut ctxt);
        generate_mipmaps(&ctxt, self.get_bind_point());
    }
}

impl TextureExt for TextureAny {
    #[inline]
    fn get_texture_id(&self) -> gl::types::GLuint {
        self.id
    }

    #[inline]
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }

    #[inline]
    fn get_bind_point(&self) -> gl::types::GLenum {
        return get_bind_point(self.ty);
    }

    fn bind_to_current(&self, ctxt: &mut CommandContext) -> gl::types::GLenum {
        let bind_point = self.get_bind_point();

        let texture_unit = ctxt.state.active_texture;
        if ctxt.state.texture_units[texture_unit as usize].texture != self.id {
            unsafe { ctxt.gl.BindTexture(bind_point, self.id) };
            ctxt.state.texture_units[texture_unit as usize].texture = self.id;
        }

        bind_point
    }
}

impl GlObject for TextureAny {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl fmt::Debug for TextureAny {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Texture #{} (dimensions: {}x{}x{}x{})", self.id,
               self.get_width(), self.get_height().unwrap_or(1), self.get_depth().unwrap_or(1),
               self.get_array_size().unwrap_or(1))
    }
}

impl Drop for TextureAny {
    fn drop(&mut self) {
        let mut ctxt = self.context.make_current();

        // removing FBOs which contain this texture
        fbo::FramebuffersContainer::purge_texture(&mut ctxt, self.id);

        // resetting the bindings
        for tex_unit in ctxt.state.texture_units.iter_mut() {
            if tex_unit.texture == self.id {
                tex_unit.texture = 0;
            }
        }

        if self.owned {
            unsafe { ctxt.gl.DeleteTextures(1, [ self.id ].as_ptr()); }
        }
    }
}

/// Represents a specific layer of an array texture and 3D textures.
#[derive(Copy, Clone)]
pub struct TextureAnyLayer<'a> {
    /// The texture.
    texture: &'a TextureAny,
    /// The layer.
    layer: u32,
}

impl<'a> TextureAnyLayer<'a> {
    /// Returns the texture.
    #[inline]
    pub fn get_texture(&self) -> &'a TextureAny {
        self.texture
    }

    /// Returns the number of samples of the texture.
    #[inline]
    pub fn get_samples(&self) -> Option<u32> {
        self.texture.get_samples()
    }

    /// Returns the layer of the texture.
    #[inline]
    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    /// Returns a structure that represents the main mipmap level of this layer of the texture.
    #[inline]
    pub fn main_level(&self) -> TextureAnyLayerMipmap<'a> {
        self.mipmap(0).unwrap()
    }

    /// Returns a structure that represents a specific mipmap of this layer of the texture.
    ///
    /// Returns `None` if out of range.
    #[inline]
    pub fn mipmap(&self, level: u32) -> Option<TextureAnyLayerMipmap<'a>> {
        if level >= self.texture.levels {
            return None;
        }

        let pow = 2u32.pow(level);

        Some(TextureAnyLayerMipmap {
            texture: self.texture,
            level: level,
            layer: self.layer,
            width: cmp::max(1, self.texture.get_width() / pow),
            height: self.texture.get_height().map(|height| cmp::max(1, height / pow)),
        })
    }
}

/// Represents a specific mipmap of a texture.
#[derive(Copy, Clone)]
pub struct TextureAnyMipmap<'a> {
    /// The texture.
    texture: &'a TextureAny,

    /// Mipmap level.
    level: u32,

    /// Width of this mipmap level.
    width: u32,
    /// Height of this mipmap level.
    height: Option<u32>,
    /// Depth of this mipmap level.
    depth: Option<u32>,
}

impl<'a> TextureAnyMipmap<'a> {
    /// Returns the width of the mipmap.
    #[inline]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the mipmap.
    #[inline]
    pub fn get_height(&self) -> Option<u32> {
        self.height
    }

    /// Returns the depth of the mipmap.
    #[inline]
    pub fn get_depth(&self) -> Option<u32> {
        self.depth
    }

    /// Computes a tuple (width, height, depth) of mipmap dimensions,
    /// using 1 for unused dimensions.
    ///
    /// In the case of 1D texture arrays, use array size as width.
    /// In the case of 2D texture arrays, use array size as depth.
    #[inline]
    fn get_mipmap_dimensions(&self) -> (u32, u32, u32) {
        let tex_depth = match self.texture.ty {
            Dimensions::Texture2dArray { array_size, .. } => array_size,
            _ => self.depth.unwrap_or(1),
        };
        let tex_height = match self.texture.ty {
            Dimensions::Texture1dArray { array_size, .. } => array_size,
            _ => self.height.unwrap_or(1),
        };
        return (self.width, tex_height, tex_depth);
    }

    /// Returns the number of samples of the texture.
    #[inline]
    pub fn get_samples(&self) -> Option<u32> {
        self.texture.get_samples()
    }

    /// Returns the texture.
    #[inline]
    pub fn get_texture(&self) -> &'a TextureAny {
        self.texture
    }

    /// Returns the level of the texture.
    #[inline]
    pub fn get_level(&self) -> u32 {
        self.level
    }

    /// Returns a structure that represents the first layer of this mipmap of the texture. All
    /// textures have a first layer.
    #[inline]
    pub fn first_layer(&self) -> TextureAnyLayerMipmap<'a> {
        self.layer(0).unwrap()
    }

    /// Returns a structure that represents a specific layer of this mipmap of the texture.
    ///
    /// Non-array textures have only one layer. The number of layers can be queried with
    /// `get_array_size`.
    ///
    /// Returns `None` if out of range.
    #[inline]
    pub fn layer(&self, layer: u32) -> Option<TextureAnyLayerMipmap<'a>> {
        if let Some(array_size) = self.texture.get_array_size() {
            if layer >= array_size {
                return None;
            }
        }

        if let Some(depth) = self.depth {
            if layer >= depth {
                return None;
            }
        }

        if layer >= 1 && self.depth.is_none() && self.texture.get_array_size().is_none() {
            return None;
        }

        Some(TextureAnyLayerMipmap {
            texture: self.texture,
            layer: layer,
            level: self.level,
            width: self.width,
            height: self.height,
        })
    }

    /// Returns the array size of the texture.
    #[inline]
    pub fn get_array_size(&self) -> Option<u32> {
        self.texture.get_array_size()
    }

    /// Uploads data to the texture from a buffer.
    ///
    /// # Panic
    ///
    /// Panics if the offsets and dimensions are outside the boundaries of the texture. Panics
    /// if the buffer is not big enough to hold the data.
    #[inline]
    pub fn raw_upload_from_pixel_buffer<P>(&self, source: BufferSlice<[P]>, x: Range<u32>,
                                           y: Range<u32>, z: Range<u32>)
                                           where P: PixelValue
    {
        self.raw_upload_from_pixel_buffer_impl(source, x, y, z, false);
    }

    /// Uploads data to the texture from a buffer. The R, G and B components are flipped.
    ///
    /// # Panic
    ///
    /// Panics if the offsets and dimensions are outside the boundaries of the texture. Panics
    /// if the buffer is not big enough to hold the data.
    #[inline]
    pub fn raw_upload_from_pixel_buffer_inverted<P>(&self, source: BufferSlice<[P]>,
                                                    x: Range<u32>, y: Range<u32>, z: Range<u32>)
                                                    where P: PixelValue
    {
        self.raw_upload_from_pixel_buffer_impl(source, x, y, z, true);
    }

    fn raw_upload_from_pixel_buffer_impl<P>(&self, source: BufferSlice<[P]>, x: Range<u32>,
                                            y: Range<u32>, z: Range<u32>, inverted: bool)
                                            where P: PixelValue
    {
        let tex_dim = self.get_mipmap_dimensions();
        assert!(x.start < tex_dim.0);
        assert!(y.start < tex_dim.1);
        assert!(z.start < tex_dim.2);
        assert!(x.end <= tex_dim.0);
        assert!(y.end <= tex_dim.1);
        assert!(z.end <= tex_dim.2);

        let width = x.end - x.start;
        let height = y.end - y.start;
        let depth = z.end - z.start;

        if source.len() < (width * height * depth) as usize {
            panic!("Buffer is too small");
        }

        let (client_format, client_type) =
            image_format::client_format_to_glenum(&self.texture.context,
                                                  ClientFormatAny::ClientFormat(P::get_format()),
                                                  self.texture.requested_format, inverted).unwrap();

        let mut ctxt = self.texture.context.make_current();

        // binds the pixel buffer
        source.prepare_and_bind_for_pixel_unpack(&mut ctxt);

        match self.texture.ty {
            Dimensions::Texture1d { .. } => {
                if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                   ctxt.extensions.gl_arb_direct_state_access
                {
                    unsafe {
                        ctxt.gl.TextureSubImage1D(self.texture.id,
                                                  self.level as gl::types::GLint,
                                                  x.start as gl::types::GLint,
                                                  width as gl::types::GLsizei,
                                                  client_format, client_type,
                                                  source.get_offset_bytes() as *const() as *const _);
                    }

                }  else if ctxt.extensions.gl_ext_direct_state_access {
                    unsafe {
                        ctxt.gl.TextureSubImage1DEXT(self.texture.id, self.texture.get_bind_point(),
                                                     self.level as gl::types::GLint,
                                                     x.start as gl::types::GLint,
                                                     width as gl::types::GLsizei,
                                                     client_format, client_type,
                                                     source.get_offset_bytes() as *const() as *const _);
                    }

                } else {
                    self.texture.bind_to_current(&mut ctxt);
                    unsafe {
                        ctxt.gl.TexSubImage1D(self.texture.get_bind_point(),
                                              self.level as gl::types::GLint,
                                              x.start as gl::types::GLint,
                                              width as gl::types::GLsizei,
                                              client_format, client_type,
                                              source.get_offset_bytes() as *const() as *const _);
                    }
                }
            },

            Dimensions::Texture1dArray { .. } | Dimensions::Texture2d { .. } |
            Dimensions::Texture2dMultisample { .. } |
            Dimensions::Texture2dMultisampleArray { .. } => {
                if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                   ctxt.extensions.gl_arb_direct_state_access
                {
                    unsafe {
                        ctxt.gl.TextureSubImage2D(self.texture.id,
                                                  self.level as gl::types::GLint,
                                                  x.start as gl::types::GLint,
                                                  y.start as gl::types::GLint,
                                                  width as gl::types::GLsizei,
                                                  height as gl::types::GLsizei,
                                                  client_format, client_type,
                                                  source.get_offset_bytes() as *const() as *const _);
                    }

                }  else if ctxt.extensions.gl_ext_direct_state_access {
                    unsafe {
                        ctxt.gl.TextureSubImage2DEXT(self.texture.id, self.texture.get_bind_point(),
                                                     self.level as gl::types::GLint,
                                                     x.start as gl::types::GLint,
                                                     y.start as gl::types::GLint,
                                                     width as gl::types::GLsizei,
                                                     height as gl::types::GLsizei,
                                                     client_format, client_type,
                                                     source.get_offset_bytes() as *const() as *const _);
                    }

                } else {
                    self.texture.bind_to_current(&mut ctxt);
                    unsafe {
                        ctxt.gl.TexSubImage2D(self.texture.get_bind_point(),
                                              self.level as gl::types::GLint,
                                              x.start as gl::types::GLint,
                                              y.start as gl::types::GLint,
                                              width as gl::types::GLsizei,
                                              height as gl::types::GLsizei,
                                              client_format, client_type,
                                              source.get_offset_bytes() as *const() as *const _);
                    }
                }
            },

            Dimensions::Texture2dArray { .. } | Dimensions::Texture3d { .. } => {
                if ctxt.version >= &Version(Api::Gl, 4, 5) ||
                   ctxt.extensions.gl_arb_direct_state_access
                {
                    unsafe {
                        ctxt.gl.TextureSubImage3D(self.texture.id,
                                                  self.level as gl::types::GLint,
                                                  x.start as gl::types::GLint,
                                                  y.start as gl::types::GLint,
                                                  z.start as gl::types::GLint,
                                                  width as gl::types::GLsizei,
                                                  height as gl::types::GLsizei,
                                                  depth as gl::types::GLsizei,
                                                  client_format, client_type,
                                                  source.get_offset_bytes() as *const() as *const _);
                    }

                }  else if ctxt.extensions.gl_ext_direct_state_access {
                    unsafe {
                        ctxt.gl.TextureSubImage3DEXT(self.texture.id, self.texture.get_bind_point(),
                                                     self.level as gl::types::GLint,
                                                     x.start as gl::types::GLint,
                                                     y.start as gl::types::GLint,
                                                     z.start as gl::types::GLint,
                                                     width as gl::types::GLsizei,
                                                     height as gl::types::GLsizei,
                                                     depth as gl::types::GLsizei,
                                                     client_format, client_type,
                                                     source.get_offset_bytes() as *const() as *const _);
                    }

                } else {
                    self.texture.bind_to_current(&mut ctxt);
                    unsafe {
                        ctxt.gl.TexSubImage3D(self.texture.get_bind_point(),
                                              self.level as gl::types::GLint,
                                              x.start as gl::types::GLint,
                                              y.start as gl::types::GLint,
                                              z.start as gl::types::GLint,
                                              width as gl::types::GLsizei,
                                              height as gl::types::GLsizei,
                                              depth as gl::types::GLsizei,
                                              client_format, client_type,
                                              source.get_offset_bytes() as *const() as *const _);
                    }
                }
            },

            Dimensions::Cubemap { .. } | Dimensions::CubemapArray { .. } => {
                panic!("Can't upload to cubemaps");     // TODO: better handling
            },
        }

        // handling synchronization for the buffer
        if let Some(fence) = source.add_fence() {
            fence.insert(&mut ctxt);
        }
    }
}

impl<'t> TextureMipmapExt for TextureAnyMipmap<'t> {
    fn upload_texture<'d, P>(&self, x_offset: u32, y_offset: u32, z_offset: u32,
                             (format, data): (ClientFormatAny, Cow<'d, [P]>), width: u32,
                             height: Option<u32>, depth: Option<u32>,
                             regen_mipmaps: bool)
                             -> Result<(), ()>   // TODO return a better Result!?
                             where P: Send + Copy + Clone + 'd
    {
        let id = self.texture.id;
        let level = self.level;

        let (is_client_compressed, data_bufsize) = (format.is_compressed(),
                                                    format.get_buffer_size(width, height, depth, None));
        let regen_mipmaps = regen_mipmaps && self.texture.levels >= 2 &&
                            self.texture.generate_mipmaps && !is_client_compressed;

        assert!(!regen_mipmaps || level == 0);  // when regen_mipmaps is true, level must be 0!
        assert!(x_offset <= self.width);
        assert!(y_offset <= self.height.unwrap_or(1));
        assert!(z_offset <= self.depth.unwrap_or(1));
        assert!(x_offset + width <= self.width);
        assert!(y_offset + height.unwrap_or(1) <= self.height.unwrap_or(1));
        assert!(z_offset + depth.unwrap_or(1) <= self.depth.unwrap_or(1));

        if data.len() * mem::size_of::<P>() != data_bufsize
        {
            panic!("Texture data size mismatch");
        }

        let (client_format, client_type) = image_format::client_format_to_glenum(&self.texture.context,
                                                                                 format,
                                                                                 self.texture.requested_format, false)
                                                                                 .map_err(|_| ())?;

        let mut ctxt = self.texture.context.make_current();

        unsafe {
            if ctxt.state.pixel_store_unpack_alignment != 1 {
                ctxt.state.pixel_store_unpack_alignment = 1;
                ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            }

            BufferAny::unbind_pixel_unpack(&mut ctxt);
            let bind_point = self.texture.bind_to_current(&mut ctxt);

            if bind_point == gl::TEXTURE_3D || bind_point == gl::TEXTURE_2D_ARRAY {
                unimplemented!();

            } else if bind_point == gl::TEXTURE_2D || bind_point == gl::TEXTURE_1D_ARRAY {
                assert!(z_offset == 0);
                // FIXME should glTexImage be used here somewhere or glTexSubImage does it just fine?
                if is_client_compressed {
                    ctxt.gl.CompressedTexSubImage2D(bind_point, level as gl::types::GLint,
                                                    x_offset as gl::types::GLint,
                                                    y_offset as gl::types::GLint,
                                                    width as gl::types::GLsizei,
                                                    height.unwrap_or(1) as gl::types::GLsizei,
                                                    client_format,
                                                    data_bufsize  as gl::types::GLsizei,
                                                    data.as_ptr() as *const _);
                } else {
                    ctxt.gl.TexSubImage2D(bind_point, level as gl::types::GLint,
                                          x_offset as gl::types::GLint,
                                          y_offset as gl::types::GLint,
                                          width as gl::types::GLsizei,
                                          height.unwrap_or(1) as gl::types::GLsizei,
                                          client_format, client_type,
                                          data.as_ptr() as *const _);
                }

            } else {
                assert!(z_offset == 0);
                assert!(y_offset == 0);

                unimplemented!();
            }

            // regenerate mipmaps if there are some
            if regen_mipmaps {
                if ctxt.version >= &Version(Api::Gl, 3, 0) {
                    ctxt.gl.GenerateMipmap(bind_point);
                } else {
                    ctxt.gl.GenerateMipmapEXT(bind_point);
                }
            }

            Ok(())
        }
    }

    fn download_compressed_data(&self) -> Option<(ClientFormatAny, Vec<u8>)> {
        let texture = self.texture;
        let level = self.level as i32;

        let mut ctxt = texture.context.make_current();

        unsafe {
            let bind_point = texture.bind_to_current(&mut ctxt);

            let mut is_compressed = 0;
            ctxt.gl.GetTexLevelParameteriv(bind_point, level, gl::TEXTURE_COMPRESSED, &mut is_compressed);
            if is_compressed != 0 {

                let mut buffer_size = 0;
                ctxt.gl.GetTexLevelParameteriv(bind_point, level, gl::TEXTURE_COMPRESSED_IMAGE_SIZE, &mut buffer_size);
                let mut internal_format = 0;
                ctxt.gl.GetTexLevelParameteriv(bind_point, level, gl::TEXTURE_INTERNAL_FORMAT, &mut internal_format);

                match ClientFormatAny::from_internal_compressed_format(internal_format as gl::types::GLenum) {
                    Some(known_format) => {
                        let mut buf = Vec::with_capacity(buffer_size as usize);
                        buf.set_len(buffer_size as usize);

                        BufferAny::unbind_pixel_pack(&mut ctxt);

                        // adjusting data alignement
                        let ptr = buf.as_ptr() as *const u8;
                        let ptr = ptr as usize;
                        if (ptr % 8) == 0 {
                        } else if (ptr % 4) == 0 && ctxt.state.pixel_store_pack_alignment != 4 {
                            ctxt.state.pixel_store_pack_alignment = 4;
                            ctxt.gl.PixelStorei(gl::PACK_ALIGNMENT, 4);
                        } else if (ptr % 2) == 0 && ctxt.state.pixel_store_pack_alignment > 2 {
                            ctxt.state.pixel_store_pack_alignment = 2;
                            ctxt.gl.PixelStorei(gl::PACK_ALIGNMENT, 2);
                        } else if ctxt.state.pixel_store_pack_alignment != 1 {
                            ctxt.state.pixel_store_pack_alignment = 1;
                            ctxt.gl.PixelStorei(gl::PACK_ALIGNMENT, 1);
                        }

                        ctxt.gl.GetCompressedTexImage(bind_point, level, buf.as_mut_ptr() as *mut _);
                        Some((known_format, buf))
                    },
                    None => None,
                }

            } else {
                None
            }
        }
    }
}

/// Represents a specific layer of a specific mipmap. This is the same as `TextureAnyImage`, except
/// for 3D textures, cubemaps and cubemap arrays.
#[derive(Copy, Clone)]
pub struct TextureAnyLayerMipmap<'a> {
    /// The texture.
    texture: &'a TextureAny,

    /// Layer for array textures, or 0 for other textures.
    layer: u32,
    /// Mipmap level.
    level: u32,

    /// Width of this layer of mipmap.
    width: u32,
    /// Height of this layer of mipmap.
    height: Option<u32>,
}

impl<'a> TextureAnyLayerMipmap<'a> {
    /// Returns the texture.
    #[inline]
    pub fn get_texture(&self) -> &'a TextureAny {
        self.texture
    }

    /// Returns the level of the texture.
    #[inline]
    pub fn get_level(&self) -> u32 {
        self.level
    }

    /// Returns the layer of the texture.
    #[inline]
    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    /// Returns the width of this texture slice.
    #[inline]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the height of this texture slice.
    #[inline]
    pub fn get_height(&self) -> Option<u32> {
        self.height
    }

    /// Returns the number of samples of the texture.
    #[inline]
    pub fn get_samples(&self) -> Option<u32> {
        self.texture.get_samples()
    }

    /// Turns this into an image.
    ///
    /// Returns `None` if `cube_layer` is `None` and the texture is a cubemap. Returns `None` if
    /// `cube_layer` is `Some` and the texture is not a cubemap.
    #[inline]
    pub fn into_image(&self, cube_layer: Option<CubeLayer>) -> Option<TextureAnyImage<'a>> {
        match (self.texture.ty, cube_layer) {
            (Dimensions::Cubemap { .. }, Some(_)) => (),
            (Dimensions::CubemapArray { .. }, Some(_)) => (),
            (Dimensions::Cubemap { .. }, None) => return None,
            (Dimensions::CubemapArray { .. }, None) => return None,
            (_, Some(_)) => return None,
            _ => ()
        };

        Some(TextureAnyImage {
            texture: self.texture,
            layer: self.layer,
            level: self.level,
            cube_layer: cube_layer,
            width: self.width,
            height: self.height,
        })
    }
}

/// Represents a specific 2D image of a texture. 1D textures are considered as having a height of 1.
#[derive(Copy, Clone)]
pub struct TextureAnyImage<'a> {
    /// The texture.
    texture: &'a TextureAny,

    /// Layer for array textures, or 0 for other textures.
    layer: u32,
    /// Mipmap level.
    level: u32,
    /// The layer of the cubemap if relevant.
    cube_layer: Option<CubeLayer>,

    /// Width of this image.
    width: u32,
    /// Height of this image.
    height: Option<u32>,
}

impl<'a> TextureAnyImage<'a> {
    /// Returns the texture.
    #[inline]
    pub fn get_texture(&self) -> &'a TextureAny {
        self.texture
    }

    /// Returns the level of the texture.
    #[inline]
    pub fn get_level(&self) -> u32 {
        self.level
    }

    /// Returns the layer of the texture.
    #[inline]
    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    /// Returns the cubemap layer of this image, or `None` if the texture is not a cubemap.
    #[inline]
    pub fn get_cubemap_layer(&self) -> Option<CubeLayer> {
        self.cube_layer
    }

    /// Returns the width of this texture slice.
    #[inline]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the height of this texture slice.
    #[inline]
    pub fn get_height(&self) -> Option<u32> {
        self.height
    }

    /// Returns the number of samples of the texture.
    #[inline]
    pub fn get_samples(&self) -> Option<u32> {
        self.texture.get_samples()
    }

    /// Reads the content of the image.
    ///
    /// # Panic
    ///
    /// - Panics if the rect is out of range.
    /// - Panics if it fails to read the texture.
    ///
    pub fn raw_read<T, P>(&self, rect: &Rect) -> T where T: Texture2dDataSink<P>, P: PixelValue {
        assert!(rect.left + rect.width <= self.width);
        assert!(rect.bottom + rect.height <= self.height.unwrap_or(1));

        let mut ctxt = self.texture.context.make_current();

        let mut data = Vec::new();
        ops::read(&mut ctxt, &fbo::RegularAttachment::Texture(*self), &rect, &mut data, false)
            .unwrap();

        T::from_raw(Cow::Owned(data), self.width, self.height.unwrap_or(1))
    }

    /// Reads the content of the image to a pixel buffer.
    ///
    /// # Panic
    ///
    /// - Panics if the rect is out of range.
    /// - Panics if the buffer is not large enough.
    /// - Panics if it fails to read the texture.
    ///
    pub fn raw_read_to_pixel_buffer<P>(&self, rect: &Rect, dest: &PixelBuffer<P>)
        where P: PixelValue
    {
        assert!(rect.left + rect.width <= self.width);
        assert!(rect.bottom + rect.height <= self.height.unwrap_or(1));
        assert!(dest.len() >= rect.width as usize * rect.height as usize);

        let size = rect.width as usize * rect.height as usize * 4;
        let mut ctxt = self.texture.context.make_current();
        ops::read(&mut ctxt, &fbo::RegularAttachment::Texture(*self), &rect, dest, false)
            .unwrap();
    }

    /// Clears the content of the texture to a specific value.
    ///
    /// # Panic
    ///
    /// Panics if `data` does not match the kind of texture. For example passing a `[i32; 4]` when
    /// using a regular (float) texture.
    ///
    pub fn raw_clear_buffer<D>(&self, data: D)
        where D: Into<ClearBufferData>
    {
        unsafe {
            let mut ctxt = self.texture.context.make_current();
            let attachment = fbo::RegularAttachment::Texture(*self);
            fbo::FramebuffersContainer::clear_buffer(&mut ctxt, &attachment, data);
        }
    }
}
