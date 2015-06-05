use std::ptr;

use buffer::BufferType;
use pixel_buffer::PixelBuffer;
use texture::ClientFormat;
use texture::PixelValue;

use fbo;

use BufferViewExt;
use Rect;
use context::CommandContext;
use gl;

/// A source for reading pixels.
pub enum Source<'a> {
    // TODO: remove the second parameter
    Attachment(&'a fbo::Attachment<'a>, &'a fbo::FramebuffersContainer),
    // TODO: use a Rust enum
    DefaultFramebuffer(gl::types::GLenum),
}

// TODO: re-enable when the second parameter is no longer needed
/*impl<'a> From<&'a fbo::Attachment> for Source<'a> {
    fn from(a: &'a fbo::Attachment) -> Source<'a> {
        Source::Attachment(a)
    }
}*/

/// A destination for reading pixels.
pub enum Destination<'a, P> where P: PixelValue {
    Memory(&'a mut Vec<P>),
    PixelBuffer(&'a PixelBuffer<P>),
    // TODO: texture with glCopyTexSubImage2D
}

impl<'a, P> From<&'a mut Vec<P>> for Destination<'a, P> where P: PixelValue {
    fn from(mem: &'a mut Vec<P>) -> Destination<'a, P> {
        Destination::Memory(mem)
    }
}

impl<'a, P> From<&'a PixelBuffer<P>> for Destination<'a, P> where P: PixelValue {
    fn from(pb: &'a PixelBuffer<P>) -> Destination<'a, P> {
        Destination::PixelBuffer(pb)
    }
}

/// Reads pixels from the source into the destination.
///
/// Panicks if the destination is not large enough.
///
/// The `(u8, u8, u8, u8)` format is guaranteed to be supported.
pub fn read<'a, S, D>(mut ctxt: &mut CommandContext, source: S, rect: &Rect, dest: D)
                      where S: Into<Source<'a>>, D: Into<Destination<'a, (u8, u8, u8, u8)>>
{
    match read_if_supported(ctxt, source, rect, dest) {
        Ok(_) => (),
        Err(_) => unreachable!(),
    }
}

/// Reads pixels from the source into the destination.
///
/// Panicks if the destination is not large enough.
pub fn read_if_supported<'a, S, D, T>(mut ctxt: &mut CommandContext, source: S, rect: &Rect,
                                      dest: D) -> Result<(), ()>
                                      where S: Into<Source<'a>>, D: Into<Destination<'a, T>>,
                                            T: PixelValue
{
    let source = source.into();
    let dest = dest.into();

    let pixels_to_read = rect.width * rect.height;

    let (fbo, read_buffer) = match source {
        Source::Attachment(attachment, framebuffer_objects) => {
            framebuffer_objects.get_framebuffer_for_reading(attachment, &mut ctxt)
        },
        Source::DefaultFramebuffer(read_buffer) => {
            (0, read_buffer)
        },
    };

    // FIXME: check if format is suppoed by ReadPixels

    let (format, gltype) = client_format_to_gl_enum(&<T as PixelValue>::get_format());

    unsafe {
        // binding framebuffer
        // FIXME: GLES2 only supports color attachment 0
        fbo::bind_framebuffer(&mut ctxt, fbo, false, true);

        // adjusting glReadBuffer
        // FIXME: handle this in `fbo`
        ctxt.gl.ReadBuffer(read_buffer);

        // reading
        match dest {
            Destination::Memory(dest) => {
                let mut buf = Vec::with_capacity(pixels_to_read as usize);

                // FIXME: correct function call
                if ctxt.state.pixel_pack_buffer_binding != 0 {
                    ctxt.gl.BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
                    ctxt.state.pixel_pack_buffer_binding = 0;
                }

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

                pixel_buffer.bind_to(&mut ctxt, BufferType::PixelPackBuffer);
                ctxt.gl.ReadPixels(rect.left as gl::types::GLint, rect.bottom as gl::types::GLint,
                                   rect.width as gl::types::GLsizei,
                                   rect.height as gl::types::GLsizei, format, gltype,
                                   ptr::null_mut());

                ::pixel_buffer::store_infos(pixel_buffer, (rect.width, rect.height));
            }
        }
    };

    Ok(())
}

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
