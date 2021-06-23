use std::ptr;
use std::fmt;
use std::error::Error;

use crate::pixel_buffer::PixelBuffer;
use crate::texture::ClientFormat;
use crate::texture::PixelValue;
use crate::image_format::{TextureFormatRequest, TextureFormat};

use crate::fbo;
use crate::fbo::FramebuffersContainer;

use crate::buffer::BufferAny;
use crate::BufferExt;
use crate::Rect;
use crate::context::CommandContext;
use crate::gl;

use crate::version::Version;
use crate::version::Api;

/// A source for reading pixels.
pub enum Source<'a> {
    /// A regular framebuffer attachment.
    Attachment(&'a fbo::RegularAttachment<'a>),
    // TODO: use a Rust enum
    DefaultFramebuffer(gl::types::GLenum),
}

impl<'a> From<&'a fbo::RegularAttachment<'a>> for Source<'a> {
    #[inline]
    fn from(a: &'a fbo::RegularAttachment<'a>) -> Source<'a> {
        Source::Attachment(a)
    }
}

/// A destination for reading pixels.
pub enum Destination<'a, P> where P: PixelValue {
    Memory(&'a mut Vec<P>),
    PixelBuffer(&'a PixelBuffer<P>),
    // TODO: texture with glCopyTexSubImage2D
}

impl<'a, P> From<&'a mut Vec<P>> for Destination<'a, P> where P: PixelValue {
    #[inline]
    fn from(mem: &'a mut Vec<P>) -> Destination<'a, P> {
        Destination::Memory(mem)
    }
}

impl<'a, P> From<&'a PixelBuffer<P>> for Destination<'a, P> where P: PixelValue {
    #[inline]
    fn from(pb: &'a PixelBuffer<P>) -> Destination<'a, P> {
        Destination::PixelBuffer(pb)
    }
}

/// Error that can happen while reading.
#[derive(Debug)]
pub enum ReadError {
    /// The implementation doesn't support converting to the requested output format.
    ///
    /// OpenGL supports every possible format, but OpenGL ES only supports `(u8, u8, u8, u8)` and
    /// an implementation-defined format.
    OutputFormatNotSupported,

    /// The implementation doesn't support reading a depth, depth-stencil or stencil attachment.
    ///
    /// OpenGL ES only supports reading from color buffers by default. There are extensions that
    /// allow reading other types of attachments.
    AttachmentTypeNotSupported,

    /// Clamping the values is not supported by the implementation.
    ClampingNotSupported,

    // TODO: context lost
}

impl fmt::Display for ReadError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ReadError::*;
        let desc = match *self {
            OutputFormatNotSupported =>
                "The implementation doesn't support converting to the requested output format",
            AttachmentTypeNotSupported =>
                "The implementation doesn't support reading a depth, depth-stencil or stencil attachment",
            ClampingNotSupported =>
                "Clamping the values is not supported by the implementation",
        };
        fmt.write_str(desc)
    }
}

impl Error for ReadError {}

/// Reads pixels from the source into the destination.
///
/// Panics if the destination is not large enough.
///
/// The `(u8, u8, u8, u8)` format is guaranteed to be supported.
// TODO: differentiate between GL_* and GL_*_INTEGER
#[inline]
pub fn read<'a, S, D, T>(mut ctxt: &mut CommandContext<'_>, source: S, rect: &Rect, dest: D,
                         clamp: bool) -> Result<(), ReadError>
                         where S: Into<Source<'a>>, D: Into<Destination<'a, T>>,
                               T: PixelValue
{
    let source = source.into();
    let dest = dest.into();
    let output_pixel_format = <T as PixelValue>::get_format();

    let pixels_to_read = rect.width * rect.height;

    // checking that the output format is supported
    // OpenGL supported everything, while OpenGL ES only supports U8U8U8U8 plus an additional
    // implementation-defined format
    if ctxt.version >= &Version(Api::GlEs, 2, 0) && output_pixel_format != ClientFormat::U8U8U8U8 {
        // TODO: GLES is guaranteed to support GL_RGBA and an implementation-defined format
        //       queried with GL_IMPLEMENTATION_COLOR_READ_FORMAT. We only handle GL_RGBA.
        return Err(ReadError::OutputFormatNotSupported);
    }

    // handling clamping
    if ctxt.version >= &Version(Api::Gl, 3, 0) {
        unsafe {
            if clamp && ctxt.state.clamp_color != gl::TRUE as gl::types::GLenum {
                ctxt.gl.ClampColor(gl::CLAMP_READ_COLOR, gl::TRUE as gl::types::GLenum);
                ctxt.state.clamp_color = gl::TRUE as gl::types::GLenum;

            } else if !clamp && ctxt.state.clamp_color != gl::FALSE as gl::types::GLenum {
                ctxt.gl.ClampColor(gl::CLAMP_READ_COLOR, gl::FALSE as gl::types::GLenum);
                ctxt.state.clamp_color = gl::FALSE as gl::types::GLenum;
            }
        }
    } else if clamp {
        return Err(ReadError::ClampingNotSupported);
    }

    // TODO: check dimensions?

    // binding framebuffer
    match source {
        Source::Attachment(attachment) => {
            unsafe { FramebuffersContainer::bind_framebuffer_for_reading(&mut ctxt, attachment) };
        },
        Source::DefaultFramebuffer(read_buffer) => {
            FramebuffersContainer::bind_default_framebuffer_for_reading(&mut ctxt, read_buffer);
        },
    };

    // determining what kind of data we are reading
    enum ReadSourceType { Color, Depth, Stencil, DepthStencil }
    let (integer, read_src_type) = match source {
        Source::Attachment(attachment) => {
            match attachment {
                fbo::RegularAttachment::Texture(ref tex) => {
                    let integer = match tex.get_texture().get_requested_format() {
                        TextureFormatRequest::Specific(TextureFormat::UncompressedIntegral(_)) => true,
                        TextureFormatRequest::Specific(TextureFormat::UncompressedUnsigned(_)) => true,
                        TextureFormatRequest::AnyIntegral => true,
                        TextureFormatRequest::AnyUnsigned => true,
                        _ => false,
                    };

                    (integer, ReadSourceType::Color)       // FIXME: wrong
                },
                fbo::RegularAttachment::RenderBuffer(ref rb) => {
                    (false, ReadSourceType::Color)       // FIXME: wrong
                },
            }
        },
        Source::DefaultFramebuffer(read_buffer) => {
            (false, ReadSourceType::Color)       // FIXME: wrong
        },
    };

    // OpenGL ES doesn't support reading from depth, stencil or depth-stencil attachments by default
    if ctxt.version >= &Version(Api::GlEs, 2, 0) {
        match read_src_type {
            ReadSourceType::Color => (),
            ReadSourceType::Depth => if !ctxt.extensions.gl_nv_read_depth {
                return Err(ReadError::AttachmentTypeNotSupported);
            },
            ReadSourceType::DepthStencil => if !ctxt.extensions.gl_nv_read_depth_stencil {
                return Err(ReadError::AttachmentTypeNotSupported);
            },
            ReadSourceType::Stencil => if !ctxt.extensions.gl_nv_read_stencil {
                return Err(ReadError::AttachmentTypeNotSupported);
            },
        }
    }

    // obtaining the client format and client type to be passed to `glReadPixels`
    let (format, gltype) = match read_src_type {
        ReadSourceType::Color => {
            client_format_to_gl_enum(&output_pixel_format, integer)
        },
        ReadSourceType::Depth => {
            unimplemented!()        // TODO:
            // TODO: NV_depth_buffer_float2
            //(gl::DEPTH_COMPONENT, )
        },
        ReadSourceType::DepthStencil => unimplemented!(),        // FIXME: only 24_8 is possible and there's no client format in the enum that corresponds to 24_8
        ReadSourceType::Stencil => {
            unimplemented!()        // TODO:
            //(gl::STENCIL_INDEX, )
        },
    };

    // reading
    unsafe {
        match dest {
            Destination::Memory(dest) => {
                let mut buf = Vec::with_capacity(pixels_to_read as usize);

                BufferAny::unbind_pixel_pack(ctxt);

                // adjusting data alignement
                let ptr = buf.as_mut_ptr() as *mut D;
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

                ctxt.gl.ReadPixels(rect.left as gl::types::GLint, rect.bottom as gl::types::GLint,
                                   rect.width as gl::types::GLsizei,
                                   rect.height as gl::types::GLsizei, format, gltype,
                                   buf.as_mut_ptr() as *mut _);
                buf.set_len(pixels_to_read as usize);

                *dest = buf;
            },

            Destination::PixelBuffer(pixel_buffer) => {
                assert!(pixel_buffer.len() >= pixels_to_read as usize);

                pixel_buffer.prepare_and_bind_for_pixel_pack(&mut ctxt);
                ctxt.gl.ReadPixels(rect.left as gl::types::GLint, rect.bottom as gl::types::GLint,
                                   rect.width as gl::types::GLsizei,
                                   rect.height as gl::types::GLsizei, format, gltype,
                                   ptr::null_mut());

                crate::pixel_buffer::store_infos(pixel_buffer, (rect.width, rect.height));
            }
        }
    };

    Ok(())
}

fn client_format_to_gl_enum(format: &ClientFormat, integer: bool)
                            -> (gl::types::GLenum, gl::types::GLenum)
{
    let (format, ty) = match *format {
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
        ClientFormat::U1U5U5U5Reversed => (gl::RGBA, gl::UNSIGNED_SHORT_1_5_5_5_REV),
        ClientFormat::U10U10U10U2 => (gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
        ClientFormat::F16 => (gl::RED, gl::HALF_FLOAT),
        ClientFormat::F16F16 => (gl::RG, gl::HALF_FLOAT),
        ClientFormat::F16F16F16 => (gl::RGB, gl::HALF_FLOAT),
        ClientFormat::F16F16F16F16 => (gl::RGBA, gl::HALF_FLOAT),
        ClientFormat::F32 => (gl::RED, gl::FLOAT),
        ClientFormat::F32F32 => (gl::RG, gl::FLOAT),
        ClientFormat::F32F32F32 => (gl::RGB, gl::FLOAT),
        ClientFormat::F32F32F32F32 => (gl::RGBA, gl::FLOAT),
    };

    let format = if integer {
        match format {
            gl::RED => gl::RED_INTEGER,
            gl::RG => gl::RG_INTEGER,
            gl::RGB => gl::RGB_INTEGER,
            gl::RGBA => gl::RGBA_INTEGER,
            _ => unreachable!()
        }
    } else {
        format
    };

    (format, ty)
}
