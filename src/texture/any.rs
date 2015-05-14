use gl;
use GlObject;

use backend::Facade;
use version::Version;
use context::Context;
use ContextExt;
use version::Api;
use Rect;

use pixel_buffer::PixelBuffer;
use image_format::{self, TextureFormatRequest};
use texture::Texture2dDataSink;
use texture::{TextureFormat, ClientFormat};
use texture::{TextureCreationError, TextureMaybeSupportedCreationError};
use texture::{get_format, InternalFormat};

use libc;
use std::fmt;
use std::mem;
use std::ptr;
use std::borrow::Cow;
use std::rc::Rc;

use ops;
use fbo;

/// A texture whose type isn't fixed at compile-time.
pub struct TextureAny {
    context: Rc<Context>,
    id: gl::types::GLuint,
    requested_format: TextureFormatRequest,
    bind_point: gl::types::GLenum,
    width: u32,
    height: Option<u32>,
    depth: Option<u32>,
    array_size: Option<u32>,

    /// Number of mipmap levels (`1` means just the main texture, `0` is not valid)
    levels: u32,
}

/// Builds a new texture.
pub fn new_texture<'a, F, P>(facade: &F, format: TextureFormatRequest,
                             data: Option<(ClientFormat, Cow<'a, [P]>)>, generate_mipmaps: bool,
                             width: u32, height: Option<u32>, depth: Option<u32>,
                             array_size: Option<u32>, samples: Option<u32>)
                             -> Result<TextureAny, TextureMaybeSupportedCreationError>
                             where P: Send + Clone + 'a, F: Facade
{
    if let Some((client_format, ref data)) = data {
        if width as usize * height.unwrap_or(1) as usize * depth.unwrap_or(1) as usize *
            array_size.unwrap_or(1) as usize * client_format.get_size() !=
            data.len() * mem::size_of::<P>()
        {
            panic!("Texture data size mismatch");
        }
    }

    // checking non-power-of-two
    if facade.get_context().get_version() < &Version(Api::Gl, 2, 0) &&
        !facade.get_context().get_extensions().gl_arb_texture_non_power_of_two
    {
        if !width.is_power_of_two() || !height.unwrap_or(2).is_power_of_two() ||
            !depth.unwrap_or(2).is_power_of_two() || !array_size.unwrap_or(2).is_power_of_two()
        {
            let ce = TextureCreationError::DimensionsNotSupported;
            return Err(TextureMaybeSupportedCreationError::CreationError(ce));
        }
    }

    let texture_type = if height.is_none() && depth.is_none() {
        assert!(samples.is_none());
        if array_size.is_none() { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }

    } else if depth.is_none() {
        match (array_size.is_some(), samples.is_some()) {
            (false, false) => gl::TEXTURE_2D,
            (true, false) => gl::TEXTURE_2D_ARRAY,
            (false, true) => gl::TEXTURE_2D_MULTISAMPLE,
            (true, true) => gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
        }

    } else {
        assert!(samples.is_none());
        gl::TEXTURE_3D
    };

    let generate_mipmaps = generate_mipmaps && match format {
        TextureFormatRequest::AnyFloatingPoint |
        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(_)) |
        TextureFormatRequest::AnyIntegral |
        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) |
        TextureFormatRequest::AnyUnsigned |
        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) => true,
        _ => false,
    };

    let texture_levels = if generate_mipmaps {
        let max = ::std::cmp::max(width, ::std::cmp::max(height.unwrap_or(1),
                             depth.unwrap_or(1))) as f32;

        match max {
            0.0 => 1,
            a => 1 + a.log2() as gl::types::GLsizei
        }

    } else {
        1
    };

    let (teximg_internal_format, storage_internal_format) =
        try!(image_format::format_request_to_glenum(facade.get_context(), data.as_ref().map(|&(c, _)| c), format));

    let (client_format, client_type) = match (&data, format) {
        (&Some((client_format, _)), f) => image_format::client_format_to_glenum(facade.get_context(), client_format, f),
        (&None, TextureFormatRequest::AnyDepth) => (gl::DEPTH_COMPONENT, gl::FLOAT),
        (&None, TextureFormatRequest::Specific(TextureFormat::DepthFormat(_))) => (gl::DEPTH_COMPONENT, gl::FLOAT),
        (&None, TextureFormatRequest::AnyDepthStencil) => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
        (&None, TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_))) => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
        (&None, _) => (gl::RGBA, gl::UNSIGNED_BYTE),
    };

    let mut ctxt = facade.get_context().make_current();

    let id = unsafe {
        let data = data;
        let data_raw = if let Some((_, ref data)) = data {
            data.as_ptr() as *const libc::c_void
        } else {
            ptr::null()
        };

        if ctxt.state.pixel_store_unpack_alignment != 1 {
            ctxt.state.pixel_store_unpack_alignment = 1;
            ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        if ctxt.state.pixel_unpack_buffer_binding != 0 {
            ctxt.state.pixel_unpack_buffer_binding = 0;
            ctxt.gl.BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
        }

        let id: gl::types::GLuint = mem::uninitialized();
        ctxt.gl.GenTextures(1, mem::transmute(&id));

        {
            ctxt.gl.BindTexture(texture_type, id);
            let act = ctxt.state.active_texture as usize;
            ctxt.state.texture_units[act].texture = id;
        }

        ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        if height.is_some() || depth.is_some() || array_size.is_some() {
            ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        }
        if depth.is_some() || array_size.is_some() {
            ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
        }
        ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        if generate_mipmaps {
            ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER,
                                  gl::LINEAR_MIPMAP_LINEAR as i32);
        } else {
            ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER,
                                  gl::LINEAR as i32);
        }

        if !generate_mipmaps && (ctxt.version >= &Version(Api::Gl, 1, 2) ||
                                 ctxt.version >= &Version(Api::GlEs, 3, 0))
        {
            ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_BASE_LEVEL, 0);
            ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MAX_LEVEL, 0);
        }

        if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
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
                ctxt.gl.TexStorage3D(texture_type, texture_levels,
                                     storage_internal_format.unwrap() as gl::types::GLenum,
                                     width, height, depth);

                if !data_raw.is_null() {
                    ctxt.gl.TexSubImage3D(texture_type, 0, 0, 0, 0, width, height, depth,
                                          client_format, client_type, data_raw);
                }

            } else {
                ctxt.gl.TexImage3D(texture_type, 0, teximg_internal_format as i32, width,
                                   height, depth, 0, client_format as u32, client_type,
                                   data_raw);
            }

        } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
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
                ctxt.gl.TexStorage2D(texture_type, texture_levels,
                                     storage_internal_format.unwrap() as gl::types::GLenum,
                                     width, height);

                if !data_raw.is_null() {
                    ctxt.gl.TexSubImage2D(texture_type, 0, 0, 0, width, height, client_format,
                                          client_type, data_raw);
                }

            } else {
                ctxt.gl.TexImage2D(texture_type, 0, teximg_internal_format as i32, width,
                                   height, 0, client_format as u32, client_type, data_raw);
            }

        } else if texture_type == gl::TEXTURE_2D_MULTISAMPLE {
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

        } else if texture_type == gl::TEXTURE_2D_MULTISAMPLE_ARRAY {
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

        } else if texture_type == gl::TEXTURE_1D {
            let mut data_raw = data_raw;

            let width = match width as gl::types::GLsizei {
                0 => { data_raw = ptr::null(); 1 },
                a => a
            };

            if storage_internal_format.is_some() && (ctxt.version >= &Version(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                ctxt.gl.TexStorage1D(texture_type, texture_levels,
                                     storage_internal_format.unwrap() as gl::types::GLenum,
                                     width);

                if !data_raw.is_null() {
                    ctxt.gl.TexSubImage1D(texture_type, 0, 0, width, client_format,
                                          client_type, data_raw);
                }

            } else {
                ctxt.gl.TexImage1D(texture_type, 0, teximg_internal_format as i32, width,
                                   0, client_format as u32, client_type, data_raw);
            }

        } else {
            unreachable!();
        }

        // only generate mipmaps for color textures
        if generate_mipmaps {
            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
               ctxt.version >= &Version(Api::GlEs, 2, 0)
            {
                ctxt.gl.GenerateMipmap(texture_type);
            } else if ctxt.extensions.gl_ext_framebuffer_object {
                ctxt.gl.GenerateMipmapEXT(texture_type);
            } else {
                unreachable!();
            }
        }

        id
    };

    Ok(TextureAny {
        context: facade.get_context().clone(),
        id: id,
        requested_format: format,
        bind_point: texture_type,
        width: width,
        height: height,
        depth: depth,
        array_size: array_size,
        levels: texture_levels as u32,
    })
}

/// Changes some parts of the texture.
pub fn upload_texture<'a, P>(tex: &TextureAny, x_offset: u32, y_offset: u32, z_offset: u32,
                             (format, data): (ClientFormat, Cow<'a, [P]>), width: u32,
                             height: Option<u32>, depth: Option<u32>, level: u32,
                             regen_mipmaps: bool)
                             where P: Send + Copy + Clone + 'a
{
    let id = tex.id;
    let bind_point = tex.bind_point;
    let regen_mipmaps = regen_mipmaps && tex.levels >= 2;

    assert!(x_offset <= tex.width);
    assert!(y_offset <= tex.height.unwrap_or(1));
    assert!(z_offset <= tex.depth.unwrap_or(1));
    assert!(x_offset + width <= tex.width);
    assert!(y_offset + height.unwrap_or(1) <= tex.height.unwrap_or(1));
    assert!(z_offset + depth.unwrap_or(1) <= tex.depth.unwrap_or(1));

    let (client_format, client_type) = image_format::client_format_to_glenum(&tex.context, format,
                                                                             tex.requested_format);

    let mut ctxt = tex.context.make_current();

    unsafe {
        if ctxt.state.pixel_store_unpack_alignment != 1 {
            ctxt.state.pixel_store_unpack_alignment = 1;
            ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        if ctxt.state.pixel_unpack_buffer_binding != 0 {
            ctxt.state.pixel_unpack_buffer_binding = 0;
            ctxt.gl.BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
        }

        {
            ctxt.gl.BindTexture(bind_point, id);
            let act = ctxt.state.active_texture as usize;
            ctxt.state.texture_units[act].texture = id;
        }

        if bind_point == gl::TEXTURE_3D || bind_point == gl::TEXTURE_2D_ARRAY {
            unimplemented!();

        } else if bind_point == gl::TEXTURE_2D || bind_point == gl::TEXTURE_1D_ARRAY {
            assert!(z_offset == 0);
            ctxt.gl.TexSubImage2D(bind_point, level as gl::types::GLint,
                                  x_offset as gl::types::GLint,
                                  y_offset as gl::types::GLint,
                                  width as gl::types::GLsizei,
                                  height.unwrap_or(1) as gl::types::GLsizei,
                                  client_format, client_type,
                                  data.as_ptr() as *const libc::c_void);

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
    }
}

/// Returns the `Context` associated with this texture.
pub fn get_context(tex: &TextureAny) -> &Rc<Context> {
    &tex.context
}

/// Returns the bind point of this texture.
///
/// The returned GLenum is guaranteed to be supported by the context.
pub fn get_bind_point(tex: &TextureAny) -> gl::types::GLenum {
    tex.bind_point
}

impl TextureAny {
    /// UNSTABLE. Reads the content of a mipmap level of the texture.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    pub fn read<T>(&self, level: u32) -> T
                   where T: Texture2dDataSink<(u8, u8, u8, u8)>
            // TODO: remove Clone for P
    {
        assert_eq!(level, 0);   // TODO:

        let attachment = fbo::Attachment::Texture {
            id: self.id,
            bind_point: self.bind_point,
            layer: 0,
            level: 0
        };

        let rect = Rect {
            bottom: 0,
            left: 0,
            width: self.width,
            height: self.height.unwrap_or(1),
        };

        let mut ctxt = self.context.make_current();

        let mut data = Vec::with_capacity(0);
        ops::read(&mut ctxt, ops::Source::Attachment(&attachment, &self.context.get_framebuffer_objects()), &rect, &mut data);
        T::from_raw(Cow::Owned(data), self.width, self.height.unwrap_or(1))
    }

    /// UNSTABLE. Reads the content of a mipmap level of the texture to a pixel buffer.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    pub fn read_to_pixel_buffer(&self, level: u32) -> PixelBuffer<(u8, u8, u8, u8)> {
        assert_eq!(level, 0);   // TODO:

        let size = self.width as usize * self.height.unwrap_or(1) as usize * 4;

        let attachment = fbo::Attachment::Texture {
            id: self.id,
            bind_point: self.bind_point,
            layer: 0,
            level: 0
        };

        let rect = Rect {
            bottom: 0,
            left: 0,
            width: self.width,
            height: self.height.unwrap_or(1),
        };

        let pb = PixelBuffer::new_empty(&self.context, size);

        let mut ctxt = self.context.make_current();
        ops::read(&mut ctxt, ops::Source::Attachment(&attachment, &self.context.get_framebuffer_objects()), &rect, &pb);
        pb
    }

    /// UNSTABLE. Returns the `Context` associated with this texture.
    pub fn get_context(&self) -> &Rc<Context> {
        &self.context
    }

    /// Returns the width of the texture.
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the texture.
    pub fn get_height(&self) -> Option<u32> {
        self.height.clone()
    }

    /// Returns the depth of the texture.
    pub fn get_depth(&self) -> Option<u32> {
        self.depth.clone()
    }

    /// Returns the array size of the texture.
    pub fn get_array_size(&self) -> Option<u32> {
        self.array_size.clone()
    }

    /// Returns the number of mipmap levels of the texture.
    pub fn get_mipmap_levels(&self) -> u32 {
        self.levels
    }

    /// Determines the internal format of this texture.
    ///
    /// Returns `None` if the backend doesn't allow querying the actual format.
    pub fn get_internal_format_if_supported(&self) -> Option<InternalFormat> {
        // TODO: cache this value in the texture
        let mut ctxt = self.context.make_current();
        get_format::get_format_if_supported(&mut ctxt, self)
    }
}

impl GlObject for TextureAny {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl fmt::Debug for TextureAny {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Texture #{} (dimensions: {}x{}x{}x{})", self.id,
               self.width, self.height.unwrap_or(1), self.depth.unwrap_or(1),
               self.array_size.unwrap_or(1))
    }
}

impl Drop for TextureAny {
    fn drop(&mut self) {
        let mut ctxt = self.context.make_current();

        // removing FBOs which contain this texture
        self.context.get_framebuffer_objects()
                    .purge_texture(self.id, &mut ctxt);

        // resetting the bindings
        for tex_unit in &mut ctxt.state.texture_units {
            if tex_unit.texture == self.id {
                tex_unit.texture = 0;
            }
        }

        unsafe { ctxt.gl.DeleteTextures(1, [ self.id ].as_ptr()); }
    }
}
