use Display;

use gl;
use GlObject;
use ToGlEnum;
use context::GlVersion;
use context::CommandContext;
use version::Api;

use pixel_buffer::PixelBuffer;
use texture::{format, Texture2dDataSink, PixelValue};
use texture::{TextureFormat, ClientFormat};
use texture::{TextureCreationError, TextureMaybeSupportedCreationError};
use texture::{UncompressedFloatFormat, UncompressedIntFormat, UncompressedUintFormat,};
use texture::DepthFormat;

use libc;
use std::fmt;
use std::mem;
use std::ptr;
use std::num::UnsignedInt;
use std::borrow::Cow;

use ops;
use fbo;

#[derive(Copy, Clone, Debug)]
pub enum TextureFormatRequest {
    Specific(TextureFormat),
    AnyFloatingPoint,
    AnyCompressed,
    AnyIntegral,
    AnyUnsigned,
    AnyDepth,
    AnyStencil,
    AnyDepthStencil,
}

pub struct TextureImplementation {
    display: Display,
    id: gl::types::GLuint,
    requested_format: TextureFormatRequest,
    format: TextureFormat,
    bind_point: gl::types::GLenum,
    width: u32,
    height: Option<u32>,
    depth: Option<u32>,
    array_size: Option<u32>,

    /// Number of mipmap levels (`1` means just the main texture, `0` is not valid)
    levels: u32,
}

impl TextureImplementation {
    /// Builds a new texture.
    pub fn new<'a, P>(display: &Display, format: TextureFormatRequest,
                      data: Option<(ClientFormat, Cow<'a, [P]>)>, generate_mipmaps: bool,
                      width: u32, height: Option<u32>, depth: Option<u32>, array_size: Option<u32>,
                      samples: Option<u32>)
                      -> Result<TextureImplementation, TextureMaybeSupportedCreationError>
                      where P: Send + Clone + 'a
    {
        use std::num::Float;

        if let Some((client_format, ref data)) = data {
            if width as usize * height.unwrap_or(1) as usize * depth.unwrap_or(1) as usize *
                array_size.unwrap_or(1) as usize * client_format.get_size() !=
                data.len() * mem::size_of::<P>()
            {
                panic!("Texture data size mismatch");
            }
        }

        // checking non-power-of-two
        if display.context.context.get_version() < &GlVersion(Api::Gl, 2, 0) &&
            !display.context.context.get_extensions().gl_arb_texture_non_power_of_two
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
            1 + (::std::cmp::max(width, ::std::cmp::max(height.unwrap_or(1),
                                 depth.unwrap_or(1))) as f32).log2() as gl::types::GLsizei
        } else {
            1
        };

        let (internal_format, can_use_texstorage) =
            try!(format_request_to_glenum(display, data.as_ref().map(|&(c, _)| c), format));

        // don't use glTexStorage for compressed textures if it has data
        let can_use_texstorage = match format {
            TextureFormatRequest::AnyCompressed |
            TextureFormatRequest::Specific(TextureFormat::CompressedFormat(_)) => {
                can_use_texstorage && data.is_none()
            },
            _ => can_use_texstorage,
        };

        let (client_format, client_type) = match (&data, format) {
            (&Some((client_format, _)), f) => client_format_to_glenum(display, client_format, f),
            (&None, TextureFormatRequest::AnyDepth) => (gl::DEPTH_COMPONENT, gl::FLOAT),
            (&None, TextureFormatRequest::Specific(TextureFormat::DepthFormat(_))) => (gl::DEPTH_COMPONENT, gl::FLOAT),
            (&None, TextureFormatRequest::AnyDepthStencil) => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
            (&None, TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_))) => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
            (&None, _) => (gl::RGBA, gl::UNSIGNED_BYTE),
        };

        let mut ctxt = display.context.context.make_current();

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

            ctxt.gl.BindTexture(texture_type, id);

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

            if !generate_mipmaps {
                ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_BASE_LEVEL, 0);
                ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MAX_LEVEL, 0);
            }

            if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
                if can_use_texstorage && (ctxt.version >= &GlVersion(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                    ctxt.gl.TexStorage3D(texture_type, texture_levels,
                                         internal_format as gl::types::GLenum,
                                         width as gl::types::GLsizei,
                                         height.unwrap() as gl::types::GLsizei,
                                         depth.or(array_size).unwrap() as gl::types::GLsizei);

                    if !data_raw.is_null() {
                        ctxt.gl.TexSubImage3D(texture_type, 0, 0, 0, 0,
                                              width as gl::types::GLsizei,
                                              height.unwrap() as gl::types::GLsizei,
                                              depth.or(array_size).unwrap() as gl::types::GLsizei,
                                              client_format, client_type, data_raw);
                    }

                } else {
                    ctxt.gl.TexImage3D(texture_type, 0, internal_format as i32, width as i32,
                                       height.unwrap() as i32,
                                       depth.or(array_size).unwrap() as i32, 0,
                                       client_format as u32, client_type, data_raw);
                }

            } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                if can_use_texstorage && (ctxt.version >= &GlVersion(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                    ctxt.gl.TexStorage2D(texture_type, texture_levels,
                                         internal_format as gl::types::GLenum,
                                         width as gl::types::GLsizei,
                                         height.or(array_size).unwrap() as gl::types::GLsizei);

                    if !data_raw.is_null() {
                        ctxt.gl.TexSubImage2D(texture_type, 0, 0, 0, width as gl::types::GLsizei,
                                              height.or(array_size).unwrap() as gl::types::GLsizei,
                                              client_format, client_type, data_raw);
                    }

                } else {
                    ctxt.gl.TexImage2D(texture_type, 0, internal_format as i32, width as i32,
                                       height.or(array_size).unwrap() as i32, 0,
                                       client_format as u32, client_type, data_raw);
                }

            } else if texture_type == gl::TEXTURE_2D_MULTISAMPLE {
                assert!(data_raw.is_null());
                if can_use_texstorage && (ctxt.version >= &GlVersion(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                    ctxt.gl.TexStorage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE,
                                                    samples.unwrap() as gl::types::GLsizei,
                                                    internal_format as gl::types::GLenum,
                                                    width as gl::types::GLsizei,
                                                    height.unwrap() as gl::types::GLsizei,
                                                    gl::TRUE);

                } else if ctxt.version >= &GlVersion(Api::Gl, 3, 2) || ctxt.extensions.gl_arb_texture_multisample {
                    ctxt.gl.TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE,
                                                  samples.unwrap() as gl::types::GLsizei,
                                                  internal_format as gl::types::GLenum,
                                                  width as gl::types::GLsizei,
                                                  height.unwrap() as gl::types::GLsizei,
                                                  gl::TRUE);

                } else {
                    unreachable!();
                }

            } else if texture_type == gl::TEXTURE_2D_MULTISAMPLE_ARRAY {
                assert!(data_raw.is_null());
                if can_use_texstorage && (ctxt.version >= &GlVersion(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                    ctxt.gl.TexStorage3DMultisample(gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
                                                    samples.unwrap() as gl::types::GLsizei,
                                                    internal_format as gl::types::GLenum,
                                                    width as gl::types::GLsizei,
                                                    height.unwrap() as gl::types::GLsizei,
                                                    array_size.unwrap() as gl::types::GLsizei,
                                                    gl::TRUE);

                } else if ctxt.version >= &GlVersion(Api::Gl, 3, 2) || ctxt.extensions.gl_arb_texture_multisample {
                    ctxt.gl.TexImage3DMultisample(gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
                                                  samples.unwrap() as gl::types::GLsizei,
                                                  internal_format as gl::types::GLenum,
                                                  width as gl::types::GLsizei,
                                                  height.unwrap() as gl::types::GLsizei,
                                                  array_size.unwrap() as gl::types::GLsizei,
                                                  gl::TRUE);

                } else {
                    unreachable!();
                }

            } else if texture_type == gl::TEXTURE_1D {
                if can_use_texstorage && (ctxt.version >= &GlVersion(Api::Gl, 4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                    ctxt.gl.TexStorage1D(texture_type, texture_levels,
                                         internal_format as gl::types::GLenum,
                                         width as gl::types::GLsizei);

                    if !data_raw.is_null() {
                        ctxt.gl.TexSubImage1D(texture_type, 0, 0, width as gl::types::GLsizei,
                                              client_format, client_type, data_raw);
                    }

                } else {
                    ctxt.gl.TexImage1D(texture_type, 0, internal_format as i32, width as i32,
                                       0, client_format as u32, client_type, data_raw);
                }

            } else {
                unreachable!();
            }

            // only generate mipmaps for color textures
            if generate_mipmaps {
                if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
                    ctxt.gl.GenerateMipmap(texture_type);
                } else {
                    ctxt.gl.GenerateMipmapEXT(texture_type);
                }
            }

            id
        };

        Ok(TextureImplementation {
            display: display.clone(),
            id: id,
            requested_format: format,
            format: unsafe { get_format(&mut ctxt, texture_type, format) },
            bind_point: texture_type,
            width: width,
            height: height,
            depth: depth,
            array_size: array_size,
            levels: texture_levels as u32,
        })
    }

    /// Reads the content of a mipmap level of the texture.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    pub fn read<P, T>(&self, level: u32) -> T
                      where P: PixelValue + Clone + Send,
                      T: Texture2dDataSink<Data = P>
            // TODO: remove Clone for P
    {
        assert_eq!(level, 0);   // TODO:

        let attachment = fbo::Attachment::Texture {
            id: self.id,
            bind_point: self.bind_point,
            layer: 0,
            level: 0
        };

        ops::read_attachment(&attachment, (self.width, self.height.unwrap_or(1)), &self.display)
    }

    /// Reads the content of a mipmap level of the texture to a pixel buffer.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    pub fn read_to_pixel_buffer<P, T>(&self, level: u32) -> PixelBuffer<T>
                                      where P: PixelValue + Clone + Send,
                                      T: Texture2dDataSink<Data = P>
            // TODO: remove Clone for P
    {
        assert_eq!(level, 0);   // TODO:

        let size = self.width as usize * self.height.unwrap_or(1) as usize *
                   <T as Texture2dDataSink>::get_preferred_formats()[0].get_size();

        let attachment = fbo::Attachment::Texture {
            id: self.id,
            bind_point: self.bind_point,
            layer: 0,
            level: 0
        };

        let mut pb = PixelBuffer::new_empty(&self.display, size);
        ops::read_attachment_to_pb(&attachment, (self.width, self.height.unwrap_or(1)),
                                   &mut pb, &self.display);
        pb
    }

    /// Changes some parts of the texture.
    pub fn upload<'a, P>(&self, x_offset: u32, y_offset: u32, z_offset: u32,
                         (format, data): (ClientFormat, Cow<'a, [P]>), width: u32,
                         height: Option<u32>, depth: Option<u32>, level: u32, regen_mipmaps: bool)
                         where P: Send + Copy + Clone + 'a
    {
        let id = self.id;
        let bind_point = self.bind_point;
        let regen_mipmaps = regen_mipmaps && self.levels >= 2;

        assert!(x_offset <= self.width);
        assert!(y_offset <= self.height.unwrap_or(1));
        assert!(z_offset <= self.depth.unwrap_or(1));
        assert!(x_offset + width <= self.width);
        assert!(y_offset + height.unwrap_or(1) <= self.height.unwrap_or(1));
        assert!(z_offset + depth.unwrap_or(1) <= self.depth.unwrap_or(1));

        let (client_format, client_type) = client_format_to_glenum(&self.display, format,
                                                                   self.requested_format);

        let mut ctxt = self.display.context.context.make_current();

        unsafe {
            if ctxt.state.pixel_store_unpack_alignment != 1 {
                ctxt.state.pixel_store_unpack_alignment = 1;
                ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            }

            if ctxt.state.pixel_unpack_buffer_binding != 0 {
                ctxt.state.pixel_unpack_buffer_binding = 0;
                ctxt.gl.BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }

            ctxt.gl.BindTexture(bind_point, id);

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
                if ctxt.version >= &GlVersion(Api::Gl, 3, 0) {
                    ctxt.gl.GenerateMipmap(bind_point);
                } else {
                    ctxt.gl.GenerateMipmapEXT(bind_point);
                }
            }
        }
    }

    /// Returns the `Display` associated with this texture.
    pub fn get_display(&self) -> &Display {
        &self.display
    }

    /// Returns the format of this texture.
    pub fn get_format(&self) -> TextureFormat {
        self.format
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
}

impl GlObject for TextureImplementation {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl fmt::Debug for TextureImplementation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Texture #{} (dimensions: {}x{}x{}x{})", self.id,
               self.width, self.height.unwrap_or(1), self.depth.unwrap_or(1),
               self.array_size.unwrap_or(1))
    }
}

impl Drop for TextureImplementation {
    fn drop(&mut self) {
        // removing FBOs which contain this texture
        self.display.context.framebuffer_objects.as_ref().unwrap()
                    .purge_texture(self.id, &self.display.context.context);

        // destroying the texture
        let ctxt = self.display.context.context.make_current();
        unsafe { ctxt.gl.DeleteTextures(1, [ self.id ].as_ptr()); }
    }
}

/// Checks that the texture format is supported and compatible with the client format.
///
/// Returns a GLenum suitable for the internal format of `glTexImage#D`. If the second element of
/// the returned tuple is `true`, it is also suitable for `glTexStorage*D`.
fn format_request_to_glenum(display: &Display, client: Option<ClientFormat>,
                            format: TextureFormatRequest)
                            -> Result<(gl::types::GLenum, bool), TextureMaybeSupportedCreationError>
{
    let version = display.context.context.get_version();
    let extensions = display.context.context.get_extensions();

    Ok(match format {
        TextureFormatRequest::AnyFloatingPoint => {
            let size = client.map(|c| c.get_num_components());

            if version >= &GlVersion(Api::Gl, 3, 0) {
                match size {
                    Some(1) => (gl::RED, false),
                    Some(2) => (gl::RG, false),
                    Some(3) => (gl::RGB, false),
                    Some(4) => (gl::RGBA, false),
                    None => (gl::RGBA, false),
                    _ => unreachable!(),
                }

            } else {
                (size.unwrap_or(4) as gl::types::GLenum, false)
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(f)) => {
            let value = f.to_glenum();
            match value {
                gl::RGB | gl::RGBA => {
                    (value, false)
                },
                gl::RGB4 | gl::RGB5 | gl::RGB8 | gl::RGB10 | gl::RGB12 | gl::RGB16 |
                gl::RGBA2 | gl::RGBA4 | gl::RGB5_A1 | gl::RGBA8 | gl::RGB10_A2 |
                gl::RGBA12 | gl::RGBA16 | gl::R3_G3_B2 => {
                    (value, true)
                },
                _ => {
                    if version >= &GlVersion(Api::Gl, 3, 0) {
                        (value, true)
                    } else {
                        return Err(TextureMaybeSupportedCreationError::CreationError(
                            TextureCreationError::UnsupportedFormat));
                    }
                }
            }
        },

        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(f)) => {
            if version >= &GlVersion(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), true)
            } else {
                return Err(TextureMaybeSupportedCreationError::CreationError(
                    TextureCreationError::UnsupportedFormat));
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(f)) => {
            if version >= &GlVersion(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), true)
            } else {
                return Err(TextureMaybeSupportedCreationError::CreationError(
                    TextureCreationError::UnsupportedFormat));
            }
        },

        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(f)) => {
            if version >= &GlVersion(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), true)
            } else {
                return Err(TextureMaybeSupportedCreationError::CreationError(
                    TextureCreationError::UnsupportedFormat));
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthFormat(f)) => {
            if version >= &GlVersion(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), true)
            } else {
                return Err(TextureMaybeSupportedCreationError::CreationError(
                    TextureCreationError::UnsupportedFormat));
            }
        },

        TextureFormatRequest::Specific(TextureFormat::StencilFormat(f)) => {
            if version >= &GlVersion(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), true)
            } else {
                return Err(TextureMaybeSupportedCreationError::CreationError(
                    TextureCreationError::UnsupportedFormat));
            }
        },

        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(f)) => {
            if version >= &GlVersion(Api::Gl, 3, 0) {    // FIXME: 
                (f.to_glenum(), true)
            } else {
                return Err(TextureMaybeSupportedCreationError::CreationError(
                    TextureCreationError::UnsupportedFormat));
            }
        },

        TextureFormatRequest::AnyCompressed => {
            let size = client.map(|c| c.get_num_components());

            if version >= &GlVersion(Api::Gl, 1, 1) {
                match size {
                    Some(1) => if version >= &GlVersion(Api::Gl, 3, 0) || extensions.gl_arb_texture_rg {
                        (gl::COMPRESSED_RED, false)
                    } else {
                        // format not supported
                        (1, false)
                    },
                    Some(2) => if version >= &GlVersion(Api::Gl, 3, 0) || extensions.gl_arb_texture_rg {
                        (gl::COMPRESSED_RG, false)
                    } else {
                        // format not supported
                        (2, false)
                    },
                    Some(3) => (gl::COMPRESSED_RGB, false),
                    Some(4) => (gl::COMPRESSED_RGBA, false),
                    None => (gl::COMPRESSED_RGBA, false),
                    _ => unreachable!(),
                }

            } else {
                // OpenGL 1.0 doesn't support compressed textures, so we use a
                // regular float format instead
                (size.unwrap_or(4) as gl::types::GLenum, false)
            }
        },

        TextureFormatRequest::AnyIntegral => {
            let size = client.map(|c| c.get_num_components());

            if version >= &GlVersion(Api::Gl, 3, 0) {
                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => (gl::R32I, true),
                    Some(2) => (gl::RG32I, true),
                    Some(3) => (gl::RGB32I, true),
                    Some(4) => (gl::RGBA32I, true),
                    None => (gl::RGBA32I, true),
                    _ => unreachable!(),
                }

            } else {
                if !extensions.gl_ext_texture_integer {
                    return Err(TextureMaybeSupportedCreationError::NotSupported);
                }

                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::R32I, true)
                    } else {
                        return Err(TextureMaybeSupportedCreationError::NotSupported);
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG32I, true)
                    } else {
                        return Err(TextureMaybeSupportedCreationError::NotSupported);
                    },
                    Some(3) => (gl::RGB32I_EXT, true),
                    Some(4) => (gl::RGBA32I_EXT, true),
                    None => (gl::RGBA32I_EXT, true),
                    _ => unreachable!(),
                }
            }
        },

        TextureFormatRequest::AnyUnsigned => {
            let size = client.map(|c| c.get_num_components());

            if version >= &GlVersion(Api::Gl, 3, 0) {
                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => (gl::R32UI, true),
                    Some(2) => (gl::RG32UI, true),
                    Some(3) => (gl::RGB32UI, true),
                    Some(4) => (gl::RGBA32UI, true),
                    None => (gl::RGBA32UI, true),
                    _ => unreachable!(),
                }

            } else {
                if !extensions.gl_ext_texture_integer {
                    return Err(TextureMaybeSupportedCreationError::NotSupported);
                }

                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::R32UI, true)
                    } else {
                        return Err(TextureMaybeSupportedCreationError::NotSupported);
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG32UI, true)
                    } else {
                        return Err(TextureMaybeSupportedCreationError::NotSupported);
                    },
                    Some(3) => (gl::RGB32UI_EXT, true),
                    Some(4) => (gl::RGBA32UI_EXT, true),
                    None => (gl::RGBA32UI_EXT, true),
                    _ => unreachable!(),
                }
            }
        },

        TextureFormatRequest::AnyDepth => {
            if version >= &GlVersion(Api::Gl, 1, 4) {
                (gl::DEPTH_COMPONENT, false)
            } else if extensions.gl_arb_depth_texture {
                (gl::DEPTH_COMPONENT, false)
            } else {
                return Err(TextureMaybeSupportedCreationError::NotSupported);
            }
        },

        TextureFormatRequest::AnyStencil => {
            if version < &GlVersion(Api::Gl, 3, 0) {
                return Err(TextureMaybeSupportedCreationError::NotSupported);
            }

            // TODO: we just request I8, but this could be more flexible
            return format_request_to_glenum(display, client,
                                     TextureFormatRequest::Specific(
                                        TextureFormat::UncompressedIntegral(
                                            ::texture::format::UncompressedIntFormat::I8)));
        },

        TextureFormatRequest::AnyDepthStencil => {
            if version >= &GlVersion(Api::Gl, 3, 0) {
                (gl::DEPTH_STENCIL, false)
            } else if extensions.gl_ext_packed_depth_stencil {
                (gl::DEPTH_STENCIL_EXT, false)
            } else {
                return Err(TextureMaybeSupportedCreationError::NotSupported);
            }
        },
    })
}

/// Checks that the client texture format is supported.
///
/// Returns two GLenums suitable for `glTexImage#D` and `glTexSubImage#D`.
fn client_format_to_glenum(display: &Display, client: ClientFormat, format: TextureFormatRequest)
                           -> (gl::types::GLenum, gl::types::GLenum)
{
    match format {
        TextureFormatRequest::AnyFloatingPoint | TextureFormatRequest::AnyCompressed |
        TextureFormatRequest::Specific(TextureFormat::UncompressedFloat(_)) |
        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(_)) =>
        {
            client_format_to_gl_enum(&client)
        },

        TextureFormatRequest::AnyIntegral |
        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) =>
        {
            client_format_to_gl_enum_int(&client).expect("Client format must \
                                                          have an integral format")
        },

        TextureFormatRequest::AnyUnsigned |
        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) =>
        {
            client_format_to_gl_enum_uint(&client).expect("Client format must \
                                                           have an integral format")
        },

        TextureFormatRequest::AnyDepth |
        TextureFormatRequest::Specific(TextureFormat::DepthFormat(_)) =>
        {
            if client != ClientFormat::F32 {
                panic!("Only ClientFormat::F32 can be used to upload on a depth texture");
            }

            (gl::DEPTH_COMPONENT, gl::FLOAT)
        }

        TextureFormatRequest::AnyStencil |
        TextureFormatRequest::Specific(TextureFormat::StencilFormat(_)) =>
        {
            let (format, ty) = client_format_to_gl_enum_int(&client).expect("Client format must \
                                                                             have an integral \
                                                                             format");

            if format != gl::RED_INTEGER {
                panic!("Can only have one component when uploading a stencil texture");
            }

            (gl::STENCIL_INDEX, ty)
        }

        TextureFormatRequest::AnyDepthStencil |
        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(_)) =>
        {
            unimplemented!();
        },
    }
}

/// Returns the two `GLenum`s corresponding to this client format.
fn client_format_to_gl_enum(format: &ClientFormat) -> (gl::types::GLenum, gl::types::GLenum) {
    match *format {
        ClientFormat::U8 => (gl::RED, gl::UNSIGNED_BYTE),
        ClientFormat::U8U8 => (gl::RG, gl::UNSIGNED_BYTE),
        ClientFormat::U8U8U8 => (gl::RGB, gl::UNSIGNED_BYTE),
        ClientFormat::U8U8U8U8 => (gl::RGBA, gl::UNSIGNED_BYTE),
        ClientFormat::I8 => (gl::RED, gl::BYTE),
        ClientFormat::I8I8 => (gl::RG, gl::BYTE),
        ClientFormat::I8I8I8 => (gl::RGB, gl::BYTE),
        ClientFormat::I8I8I8I8 => (gl::RGBA, gl::BYTE),
        ClientFormat::U16 => (gl::RED, gl::UNSIGNED_SHORT),
        ClientFormat::U16U16 => (gl::RG, gl::UNSIGNED_SHORT),
        ClientFormat::U16U16U16 => (gl::RGB, gl::UNSIGNED_SHORT),
        ClientFormat::U16U16U16U16 => (gl::RGBA, gl::UNSIGNED_SHORT),
        ClientFormat::I16 => (gl::RED, gl::SHORT),
        ClientFormat::I16I16 => (gl::RG, gl::SHORT),
        ClientFormat::I16I16I16 => (gl::RGB, gl::SHORT),
        ClientFormat::I16I16I16I16 => (gl::RGBA, gl::SHORT),
        ClientFormat::U32 => (gl::RED, gl::UNSIGNED_INT),
        ClientFormat::U32U32 => (gl::RG, gl::UNSIGNED_INT),
        ClientFormat::U32U32U32 => (gl::RGB, gl::UNSIGNED_INT),
        ClientFormat::U32U32U32U32 => (gl::RGBA, gl::UNSIGNED_INT),
        ClientFormat::I32 => (gl::RED, gl::INT),
        ClientFormat::I32I32 => (gl::RG, gl::INT),
        ClientFormat::I32I32I32 => (gl::RGB, gl::INT),
        ClientFormat::I32I32I32I32 => (gl::RGBA, gl::INT),
        ClientFormat::U3U3U2 => (gl::RGB, gl::UNSIGNED_BYTE_3_3_2),
        ClientFormat::U5U6U5 => (gl::RGB, gl::UNSIGNED_SHORT_5_6_5),
        ClientFormat::U4U4U4U4 => (gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4),
        ClientFormat::U5U5U5U1 => (gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1),
        ClientFormat::U10U10U10U2 => (gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
        ClientFormat::F16 => (gl::RED, gl::HALF_FLOAT),
        ClientFormat::F16F16 => (gl::RG, gl::HALF_FLOAT),
        ClientFormat::F16F16F16 => (gl::RGB, gl::HALF_FLOAT),
        ClientFormat::F16F16F16F16 => (gl::RGBA, gl::HALF_FLOAT),
        ClientFormat::F32 => (gl::RED, gl::FLOAT),
        ClientFormat::F32F32 => (gl::RG, gl::FLOAT),
        ClientFormat::F32F32F32 => (gl::RGB, gl::FLOAT),
        ClientFormat::F32F32F32F32 => (gl::RGBA, gl::FLOAT),
    }
}

/// Returns the two `GLenum`s corresponding to this client format for the "signed integer" format,
/// if possible
fn client_format_to_gl_enum_int(format: &ClientFormat)
                                -> Option<(gl::types::GLenum, gl::types::GLenum)>
{
    let (components, format) = client_format_to_gl_enum(format);

    let components = match components {
        gl::RED => gl::RED_INTEGER,
        gl::RG => gl::RG_INTEGER,
        gl::RGB => gl::RGB_INTEGER,
        gl::RGBA => gl::RGBA_INTEGER,
        _ => return None
    };

    match format {
        gl::BYTE => (),
        gl::SHORT => (),
        gl::INT => (),
        _ => return None
    };

    Some((components, format))
}

/// Returns the two `GLenum`s corresponding to this client format for the "unsigned integer" format,
/// if possible
fn client_format_to_gl_enum_uint(format: &ClientFormat)
                                 -> Option<(gl::types::GLenum, gl::types::GLenum)>
{
    let (components, format) = client_format_to_gl_enum(format);

    let components = match components {
        gl::RED => gl::RED_INTEGER,
        gl::RG => gl::RG_INTEGER,
        gl::RGB => gl::RGB_INTEGER,
        gl::RGBA => gl::RGBA_INTEGER,
        _ => return None
    };

    match format {
        gl::UNSIGNED_BYTE => (),
        gl::UNSIGNED_SHORT => (),
        gl::UNSIGNED_INT => (),
        gl::UNSIGNED_BYTE_3_3_2 => (),
        gl::UNSIGNED_SHORT_5_6_5 => (),
        gl::UNSIGNED_SHORT_4_4_4_4 => (),
        gl::UNSIGNED_SHORT_5_5_5_1 => (),
        gl::UNSIGNED_INT_10_10_10_2 => (),
        _ => return None
    };

    Some((components, format))
}

/// Returns the `UncompressedFloatFormat` most suitable for the `ClientFormat`.
fn to_float_internal_format(format: &ClientFormat) -> Option<format::UncompressedFloatFormat> {
    use texture::format::UncompressedFloatFormat;

    match *format {
        ClientFormat::U8 => Some(UncompressedFloatFormat::U8),
        ClientFormat::U8U8 => Some(UncompressedFloatFormat::U8U8),
        ClientFormat::U8U8U8 => Some(UncompressedFloatFormat::U8U8U8),
        ClientFormat::U8U8U8U8 => Some(UncompressedFloatFormat::U8U8U8U8),
        ClientFormat::I8 => Some(UncompressedFloatFormat::I8),
        ClientFormat::I8I8 => Some(UncompressedFloatFormat::I8I8),
        ClientFormat::I8I8I8 => Some(UncompressedFloatFormat::I8I8I8),
        ClientFormat::I8I8I8I8 => Some(UncompressedFloatFormat::I8I8I8I8),
        ClientFormat::U16 => Some(UncompressedFloatFormat::U16),
        ClientFormat::U16U16 => Some(UncompressedFloatFormat::U16U16),
        ClientFormat::U16U16U16 => None,
        ClientFormat::U16U16U16U16 => Some(UncompressedFloatFormat::U16U16U16U16),
        ClientFormat::I16 => Some(UncompressedFloatFormat::I16),
        ClientFormat::I16I16 => Some(UncompressedFloatFormat::I16I16),
        ClientFormat::I16I16I16 => Some(UncompressedFloatFormat::I16I16I16),
        ClientFormat::I16I16I16I16 => None,
        ClientFormat::U32 => None,
        ClientFormat::U32U32 => None,
        ClientFormat::U32U32U32 => None,
        ClientFormat::U32U32U32U32 => None,
        ClientFormat::I32 => None,
        ClientFormat::I32I32 => None,
        ClientFormat::I32I32I32 => None,
        ClientFormat::I32I32I32I32 => None,
        ClientFormat::U3U3U2 => None,
        ClientFormat::U5U6U5 => None,
        ClientFormat::U4U4U4U4 => Some(UncompressedFloatFormat::U4U4U4U4),
        ClientFormat::U5U5U5U1 => Some(UncompressedFloatFormat::U5U5U5U1),
        ClientFormat::U10U10U10U2 => Some(UncompressedFloatFormat::U10U10U10U2),
        ClientFormat::F16 => Some(UncompressedFloatFormat::F16),
        ClientFormat::F16F16 => Some(UncompressedFloatFormat::F16F16),
        ClientFormat::F16F16F16 => Some(UncompressedFloatFormat::F16F16F16),
        ClientFormat::F16F16F16F16 => Some(UncompressedFloatFormat::F16F16F16F16),
        ClientFormat::F32 => Some(UncompressedFloatFormat::F32),
        ClientFormat::F32F32 => Some(UncompressedFloatFormat::F32F32),
        ClientFormat::F32F32F32 => Some(UncompressedFloatFormat::F32F32F32),
        ClientFormat::F32F32F32F32 => Some(UncompressedFloatFormat::F32F32F32F32),
    }
}

/// Finds the `TextureFormat` of a texture.
///
/// Technically we find the format of the level 0, but in glium textures always have the same
/// format at all levels.
unsafe fn get_format(ctxt: &mut CommandContext, bind_point: gl::types::GLenum,
                     requested: TextureFormatRequest) -> TextureFormat
{
    match requested {
        TextureFormatRequest::Specific(format) => {
            return format;
        },
        _ => ()
    };

    if !(ctxt.version >= &GlVersion(Api::Gl, 1, 1)) &&
        !(ctxt.version >= &GlVersion(Api::GlEs, 3, 0))
    {
        // glGetTexLevelParameteriv not available
        unimplemented!();
    }

    // FIXME: handle compressed formats

    let (red_bits, green_bits, blue_bits, alpha_bits, depth_bits) = {
        let mut red = mem::uninitialized();
        ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_RED_SIZE, &mut red);

        let mut green = mem::uninitialized();
        ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_GREEN_SIZE, &mut green);

        let mut blue = mem::uninitialized();
        ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_BLUE_SIZE, &mut blue);

        let mut alpha = mem::uninitialized();
        ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_ALPHA_SIZE, &mut alpha);

        let mut depth = mem::uninitialized();
        ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_DEPTH_SIZE, &mut depth);

        (red, green, blue, alpha, depth)
    };

    let (red_type, green_type, blue_type, alpha_type, depth_type) = {
        if ctxt.version >= &GlVersion(Api::Gl, 3, 0) || ctxt.version >= &GlVersion(Api::GlEs, 3, 0) {
            let mut red = mem::uninitialized();
            ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_RED_TYPE, &mut red);

            let mut green = mem::uninitialized();
            ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_GREEN_TYPE, &mut green);

            let mut blue = mem::uninitialized();
            ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_BLUE_TYPE, &mut blue);

            let mut alpha = mem::uninitialized();
            ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_ALPHA_TYPE, &mut alpha);

            let mut depth = mem::uninitialized();
            ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_DEPTH_TYPE, &mut depth);

            (red as gl::types::GLenum, green as gl::types::GLenum, blue as gl::types::GLenum,
                alpha as gl::types::GLenum, depth as gl::types::GLenum)

        } else {
            // FIXME: handle ARB_texture_rg and similar
            (gl::FLOAT, gl::FLOAT, gl::FLOAT, gl::FLOAT, gl::FLOAT)
        }
    };

    match (red_bits, red_type, green_bits, green_type, blue_bits, blue_type,
           alpha_bits, alpha_type, depth_bits, depth_type)
    {
        (8, t, 0, _, 0, _, 0, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8),
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (8, t, 8, _, 0, _, 0, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8),
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (8, t, 8, _, 8, _, 0, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8I8),
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8U8),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8I8),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8U8),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (8, t, 8, _, 8, _, 8, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8I8I8),
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8U8U8),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8I8I8),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8U8U8),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (16, t, 0, _, 0, _, 0, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16),
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16),
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (16, t, 16, _, 0, _, 0, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16I16),
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16U16),
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (16, t, 16, _, 16, _, 0, _, 0, _) => {
            match t {
                gl::SIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16I16I16),
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16F16),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16I16),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16U16),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (16, t, 16, _, 16, _, 16, _, 0, _) => {
            match t {
                gl::UNSIGNED_NORMALIZED => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16U16U16U16),
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16F16F16),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16I16I16),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16U16U16),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (32, t, 0, _, 0, _, 0, _, 0, _) => {
            match t {
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (32, t, 32, _, 0, _, 0, _, 0, _) => {
            match t {
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (32, t, 32, _, 32, _, 0, _, 0, _) => {
            match t {
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32F32),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32I32),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32U32),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (32, t, 32, _, 32, _, 32, _, 0, _) => {
            match t {
                gl::FLOAT => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32F32F32),
                gl::INT => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32I32I32),
                gl::UNSIGNED_INT => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32U32U32),
                _ => unreachable!("Unexpected type: 0x{:x}", t)
            }
        },
        (3, gl::UNSIGNED_NORMALIZED, 3, _, 2, _, 0, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U3U3U2)
        },
        (4, gl::UNSIGNED_NORMALIZED, 4, _, 4, _, 0, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U4U4U4)
        },
        (5, gl::UNSIGNED_NORMALIZED, 5, _, 5, _, 0, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U5U5U5)
        },
        (10, gl::UNSIGNED_NORMALIZED, 10, _, 10, _, 0, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U10U10U10)
        },
        (12, gl::UNSIGNED_NORMALIZED, 12, _, 12, _, 0, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U12U12U12)
        },
        (2, gl::UNSIGNED_NORMALIZED, 2, _, 2, _, 2, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U2U2U2U2)
        },
        (4, gl::UNSIGNED_NORMALIZED, 4, _, 4, _, 4, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U4U4U4U4)
        },
        (5, gl::UNSIGNED_NORMALIZED, 5, _, 5, _, 1, _, 0, _) => {
            TextureFormat::UncompressedFloat(UncompressedFloatFormat::U5U5U5U1)
        },
        (a, b, c, d, e, f, g, h, i, j) => panic!("Unexpected texture format: \
                                                  ({}/{} {}/{} {}/{} {}/{} {}/{})", 
                                                  a, b, c, d, e, f, g, h, i, j)
    }
/*
    match (format, requested) {
        (gl::R8, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8),
        (gl::R8_SNORM, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8),
        (gl::R16, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16),
        (gl::R16_SNORM, TextureFormatRequest::AnyDepth) => TextureFormat::DepthFormat(DepthFormat::I16),
        (gl::R16_SNORM, TextureFormatRequest::AnyFloatingPoint) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16),
        (gl::RG8, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8),
        (gl::RG8_SNORM, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8),
        (gl::RG16, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16U16),
        (gl::RG16_SNORM, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16I16),
        (gl::R3_G3_B2, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U3U3U2),
        (gl::RGB4, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U4U4U4),     
        (gl::RGB5, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U5U5U5),
        (gl::RGB8, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8U8),
        (gl::RGB8_SNORM, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8I8),
        (gl::RGB10, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U10U10U10),
        (gl::RGB12, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U12U12U12),
        (gl::RGB16_SNORM, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I16I16I16),
        (gl::RGBA2, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U2U2U2U2),  
        (gl::RGBA4, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U4U4U4U4), 
        (gl::RGB5_A1, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U5U5U5U1),
        (gl::RGBA8, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U8U8U8U8),
        (gl::RGBA8_SNORM, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::I8I8I8I8),
        (gl::RGB10_A2, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U10U10U10U2),
        (gl::RGB10_A2UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U10U10U10U2),
        (gl::RGBA12, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U12U12U12U12),
        (gl::RGBA16, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::U16U16U16U16),
        (gl::SRGB8    gl::RGB  8   8   8        
        (gl::SRGB8_ALPHA8     gl::RGBA     8   8   8   8    
        (gl::R16F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16),
        (gl::RG16F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16),
        (gl::RGB16F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16F16),
        (gl::RGBA16F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F16F16F16F16),
        (gl::R32F, TextureFormatRequest::AnyDepth) => TextureFormat::DepthFormat(DepthFormat::F32),
        (gl::R32F, TextureFormatRequest::AnyFloatingPoint) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32),
        (gl::RG32F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32),
        (gl::RGB32F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32F32),
        (gl::RGBA32F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F32F32F32F32),
        (gl::R11F_G11F_B10F, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F11F11F10),
        (gl::RGB9_E5, _) => TextureFormat::UncompressedFloat(UncompressedFloatFormat::F9F9F9),
        (gl::R8I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8),
        (gl::R8UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8),
        (gl::R16I, TextureFormatRequest::AnyDepth) => TextureFormat::DepthFormat(DepthFormat::I16),
        (gl::R16I, TextureFormatRequest::AnyFloatingPoint) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16),
        (gl::R16UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16),
        (gl::R32I, TextureFormatRequest::AnyDepth) => TextureFormat::DepthFormat(DepthFormat::I32),
        (gl::R32I, TextureFormatRequest::AnyFloatingPoint) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32),
        (gl::R32UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32),
        (gl::RG8I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8),
        (gl::RG8UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8),
        (gl::RG16I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16),
        (gl::RG16UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16),
        (gl::RG32I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32),
        (gl::RG32UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32),
        (gl::RGB8I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8I8),
        (gl::RGB8UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8U8),
        (gl::RGB16I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16I16),
        (gl::RGB16UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16U16),
        (gl::RGB32I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32I32),
        (gl::RGB32UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32U32),
        (gl::RGBA8I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I8I8I8I8),
        (gl::RGBA8UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U8U8U8U8),
        (gl::RGBA16I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I16I16I16I16),
        (gl::RGBA16UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U16U16U16U16),
        (gl::RGBA32I, _) => TextureFormat::UncompressedIntegral(UncompressedIntFormat::I32I32I32I32),
        (gl::RGBA32UI, _) => TextureFormat::UncompressedUnsigned(UncompressedUintFormat::U32U32U32U32),
        (_, TextureFormatRequest::Specific(_)) => unreachable!(),
        (e, rq) => panic!("Unimplemented tex get_format ; glenum value: {}, requested: {:?}", e, rq),
    }*/
}
