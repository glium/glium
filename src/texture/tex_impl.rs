use Display;

use gl;
use GlObject;
use ToGlEnum;
use context::GlVersion;

use pixel_buffer::PixelBuffer;
use texture::{format, Texture2dData, PixelValue, TextureFormat, ClientFormat};

use libc;
use std::fmt;
use std::mem;
use std::ptr;
use std::sync::mpsc::channel;

use ops;
use fbo;

#[derive(Copy, Clone)]
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
    bind_point: gl::types::GLenum,
    width: u32,
    height: Option<u32>,
    depth: Option<u32>,
    array_size: Option<u32>,
}

impl TextureImplementation {
    /// Builds a new texture.
    pub fn new<P>(display: &Display, format: TextureFormatRequest,
                  data: Option<(ClientFormat, Vec<P>)>, width: u32, height: Option<u32>,
                  depth: Option<u32>, array_size: Option<u32>)
                  -> TextureImplementation where P: Send
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

        let texture_type = if height.is_none() && depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }
        } else if depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_2D } else { gl::TEXTURE_2D_ARRAY }
        } else {
            gl::TEXTURE_3D
        };

        let generate_mipmaps = match format {
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
            format_request_to_glenum(display, data.as_ref().map(|&(c, _)| c), format);

        // don't use glTexStorage for compressed textures if it has data
        let can_use_texstorage = match format {
            TextureFormatRequest::AnyCompressed |
            TextureFormatRequest::Specific(TextureFormat::CompressedFormat(_)) => {
                can_use_texstorage && data.is_none()
            },
            _ => can_use_texstorage,
        };

        let (client_format, client_type) = match data {
            Some((client_format, _)) => client_format_to_glenum(display, client_format, format),
            None => (gl::RGBA, gl::UNSIGNED_BYTE),
        };

        let (tx, rx) = channel();
        display.context.context.exec(move |: ctxt| {
            unsafe {
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
                ctxt.gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as i32);

                if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
                    if can_use_texstorage && (ctxt.version >= &GlVersion(4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                        ctxt.gl.TexStorage3D(texture_type, texture_levels,
                                             internal_format as gl::types::GLenum,
                                             width as gl::types::GLsizei,
                                             height.unwrap() as gl::types::GLsizei,
                                             depth.unwrap() as gl::types::GLsizei);

                        if !data_raw.is_null() {
                            ctxt.gl.TexSubImage3D(texture_type, 0, 0, 0, 0,
                                                  width as gl::types::GLsizei,
                                                  height.unwrap() as gl::types::GLsizei,
                                                  depth.unwrap() as gl::types::GLsizei,
                                                  client_format, client_type, data_raw);
                        }

                    } else {
                        ctxt.gl.TexImage3D(texture_type, 0, internal_format as i32, width as i32,
                                           height.unwrap() as i32,
                                           if let Some(d) = depth { d } else {
                                               array_size.unwrap_or(1)
                                           } as i32, 0,
                                           client_format as u32, client_type, data_raw);
                    }

                } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                    if can_use_texstorage && (ctxt.version >= &GlVersion(4, 2) || ctxt.extensions.gl_arb_texture_storage) {
                        ctxt.gl.TexStorage2D(texture_type, texture_levels,
                                             internal_format as gl::types::GLenum,
                                             width as gl::types::GLsizei,
                                             height.unwrap() as gl::types::GLsizei);

                        if !data_raw.is_null() {
                            ctxt.gl.TexSubImage2D(texture_type, 0, 0, 0, width as gl::types::GLsizei,
                                                  height.unwrap() as gl::types::GLsizei,
                                                  client_format, client_type, data_raw);
                        }

                    } else {
                        ctxt.gl.TexImage2D(texture_type, 0, internal_format as i32, width as i32,
                                           height.unwrap() as i32, 0, client_format as u32,
                                           client_type, data_raw);
                    }

                } else {
                    if can_use_texstorage && (ctxt.version >= &GlVersion(4, 2) || ctxt.extensions.gl_arb_texture_storage) {
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
                }

                // only generate mipmaps for color textures
                if generate_mipmaps {
                    if ctxt.version >= &GlVersion(3, 0) {
                        ctxt.gl.GenerateMipmap(texture_type);
                    } else {
                        ctxt.gl.GenerateMipmapEXT(texture_type);
                    }
                }

                tx.send(id).unwrap();
            }
        });

        TextureImplementation {
            display: display.clone(),
            id: rx.recv().unwrap(),
            bind_point: texture_type,
            width: width,
            height: height,
            depth: depth,
            array_size: array_size,
        }
    }

    /// Reads the content of a mipmap level of the texture.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    pub fn read<P, T>(&self, level: u32) -> T
                      where P: PixelValue + Clone + Send,
                      T: Texture2dData<Data = P>
            // TODO: remove Clone for P
    {
        assert_eq!(level, 0);   // TODO:
        ops::read_attachment(&fbo::Attachment::Texture(self.id), (self.width,
                             self.height.unwrap_or(1)), &self.display)
    }

    /// Reads the content of a mipmap level of the texture to a pixel buffer.
    // TODO: this function only works for level 0 right now
    //       width/height need adjustements
    pub fn read_to_pixel_buffer<P, T>(&self, level: u32) -> PixelBuffer<T>
                                      where P: PixelValue + Clone + Send,
                                      T: Texture2dData<Data = P>
            // TODO: remove Clone for P
    {
        assert_eq!(level, 0);   // TODO:

        let size = self.width as usize * self.height.unwrap_or(1) as usize *
                   <T as Texture2dData>::get_format().get_size();

        let mut pb = PixelBuffer::new_empty(&self.display, size);
        ops::read_attachment_to_pb(&fbo::Attachment::Texture(self.id), (self.width,
                                   self.height.unwrap_or(1)), &mut pb, &self.display);
        pb
    }

    /// Returns the `Display` associated with this texture.
    pub fn get_display(&self) -> &Display {
        &self.display
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
        let id = self.id.clone();
        self.display.context.context.exec(move |: ctxt| {
            unsafe { ctxt.gl.DeleteTextures(1, [ id ].as_ptr()); }
        });
    }
}

/// Checks that the texture format is supported and compatible with the client format.
///
/// Returns a GLenum suitable for the internal format of `glTexImage#D`. If the second element of
/// the returned tuple is `true`, it is also suitable for `glTexStorage*D`.
fn format_request_to_glenum(display: &Display, client: Option<ClientFormat>,
                            format: TextureFormatRequest)
                            -> (gl::types::GLenum, bool)
{
    let version = display.context.context.get_version();
    let extensions = display.context.context.get_extensions();

    match format {
        TextureFormatRequest::AnyFloatingPoint => {
            let size = client.map(|c| c.get_num_components());

            if version >= &GlVersion(3, 0) {
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
                    assert!(version >= &GlVersion(3, 0));       // FIXME: 
                    (value, true)
                }
            }
        },
        TextureFormatRequest::Specific(TextureFormat::CompressedFormat(f)) => {
            assert!(version >= &GlVersion(3, 0));       // FIXME: 
            (f.to_glenum(), true)
        },
        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(f)) => {
            assert!(version >= &GlVersion(3, 0));       // FIXME: 
            (f.to_glenum(), true)
        },
        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(f)) => {
            assert!(version >= &GlVersion(3, 0));       // FIXME: 
            (f.to_glenum(), true)
        },
        TextureFormatRequest::Specific(TextureFormat::DepthFormat(f)) => {
            assert!(version >= &GlVersion(3, 0));       // FIXME: 
            (f.to_glenum(), true)
        },
        TextureFormatRequest::Specific(TextureFormat::StencilFormat(f)) => {
            assert!(version >= &GlVersion(3, 0));       // FIXME: 
            (f.to_glenum(), true)
        },
        TextureFormatRequest::Specific(TextureFormat::DepthStencilFormat(f)) => {
            assert!(version >= &GlVersion(3, 0));       // FIXME: 
            (f.to_glenum(), true)
        },

        TextureFormatRequest::AnyCompressed => {
            let size = client.map(|c| c.get_num_components());

            if version >= &GlVersion(1, 1) {
                match size {
                    Some(1) => if version >= &GlVersion(3, 0) || extensions.gl_arb_texture_rg {
                        (gl::COMPRESSED_RED, false)
                    } else {
                        // format not supported
                        (1, false)
                    },
                    Some(2) => if version >= &GlVersion(3, 0) || extensions.gl_arb_texture_rg {
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

            if version >= &GlVersion(3, 0) {
                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => (gl::R32I, true),
                    Some(2) => (gl::RG32I, true),
                    Some(3) => (gl::RGB32I, true),
                    Some(4) => (gl::RGBA32I, true),
                    None => (gl::RGBA32I, true),
                    _ => unreachable!(),
                }

            } else {
                assert!(extensions.gl_ext_texture_integer, "Integral textures are not supported \
                                                            by the backend");

                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::R32I, true)
                    } else {
                        panic!("The backend doesn't support one-component integral textures");
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG32I, true)
                    } else {
                        panic!("The backend doesn't support one-component integral textures");
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

            if version >= &GlVersion(3, 0) {
                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => (gl::R32UI, true),
                    Some(2) => (gl::RG32UI, true),
                    Some(3) => (gl::RGB32UI, true),
                    Some(4) => (gl::RGBA32UI, true),
                    None => (gl::RGBA32UI, true),
                    _ => unreachable!(),
                }

            } else {
                assert!(extensions.gl_ext_texture_integer, "Integral textures are not supported \
                                                            by the backend");

                match size {  // FIXME: choose between 8, 16 and 32 depending on the client format
                    Some(1) => if extensions.gl_arb_texture_rg {
                        (gl::R32UI, true)
                    } else {
                        panic!("The backend doesn't support one-component integral textures");
                    },
                    Some(2) => if extensions.gl_arb_texture_rg {
                        (gl::RG32UI, true)
                    } else {
                        panic!("The backend doesn't support one-component integral textures");
                    },
                    Some(3) => (gl::RGB32UI_EXT, true),
                    Some(4) => (gl::RGBA32UI_EXT, true),
                    None => (gl::RGBA32UI_EXT, true),
                    _ => unreachable!(),
                }
            }
        },

        TextureFormatRequest::AnyDepth => {
            assert!(version >= &GlVersion(1, 1), "OpenGL 1.0 doesn't support depth textures");
            (gl::DEPTH_COMPONENT, false)
        },

        TextureFormatRequest::AnyStencil => {
            // TODO: we just request I8, but this could be more flexible
            format_request_to_glenum(display, client,
                                     TextureFormatRequest::Specific(
                                        TextureFormat::UncompressedIntegral(
                                            ::texture::format::UncompressedIntFormat::I8)))
        },

        TextureFormatRequest::AnyDepthStencil => {
            assert!(version >= &GlVersion(3, 0));
            (gl::DEPTH_STENCIL, false)
        },
    }
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
