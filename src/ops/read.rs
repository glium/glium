use std::ptr;
use std::sync::mpsc;

use Display;
use pixel_buffer::{self, PixelBuffer};
use texture::ClientFormat;

use fbo;
use texture;

use GlObject;
use libc;
use context;
use gl;

pub fn read_attachment<P, T>(attachment: &fbo::Attachment, dimensions: (u32, u32),
                             display: &Display) -> T          // TODO: remove Clone for P
                             where P: texture::PixelValue + Clone + Send,
                             T: texture::Texture2dDataSink<Data = P>
{
    let (fbo, atch) = display.context.framebuffer_objects.as_ref().unwrap()
                             .get_framebuffer_for_reading(attachment, &display.context.context);
    read_impl(fbo, atch, dimensions, None, &display.context.context).unwrap()
}

/// Panics if the pixel buffer is not big enough.
pub fn read_attachment_to_pb<P, T>(attachment: &fbo::Attachment, dimensions: (u32, u32),
                                   dest: &mut PixelBuffer<T>, display: &Display)          // TODO: remove Clone for P
                                   where P: texture::PixelValue + Clone + Send,
                                   T: texture::Texture2dDataSink<Data = P>
{
    let (fbo, atch) = display.context.framebuffer_objects.as_ref().unwrap()
                             .get_framebuffer_for_reading(attachment, &display.context.context);
    read_impl(fbo, atch, dimensions, Some(dest), &display.context.context);
}

pub fn read_from_default_fb<P, T>(attachment: gl::types::GLenum, display: &Display) -> T          // TODO: remove Clone for P
                                  where P: texture::PixelValue + Clone + Send,
                                  T: texture::Texture2dDataSink<Data = P>
{
    let (w, h) = display.get_framebuffer_dimensions();
    let (w, h) = (w as u32, h as u32);      // TODO: remove this conversion
    read_impl(0, attachment, (w, h), None, &display.context.context).unwrap()
}

/// Panics if the pixel buffer is not big enough.
pub fn read_from_default_fb_to_pb<P, T>(attachment: gl::types::GLenum,
                                        dest: &mut PixelBuffer<T>, display: &Display)          // TODO: remove Clone for P
                                        where P: texture::PixelValue + Clone + Send,
                                        T: texture::Texture2dDataSink<Data = P>
{
    let (w, h) = display.get_framebuffer_dimensions();
    read_impl(0, attachment, (w, h), Some(dest), &display.context.context);
}

fn read_impl<P, T>(fbo: gl::types::GLuint, readbuffer: gl::types::GLenum,
                   dimensions: (u32, u32), target: Option<&mut PixelBuffer<T>>,
                   context: &context::Context) -> Option<T>          // TODO: remove Clone for P
                   where P: texture::PixelValue + Clone + Send,
                   T: texture::Texture2dDataSink<Data = P>
{
    use std::mem;

    let pixels_count = dimensions.0 * dimensions.1;

    let chosen_format = <T as texture::Texture2dDataSink>::get_preferred_formats()[0];
    let pixels_size = chosen_format.get_size();
    let (format, gltype) = client_format_to_gl_enum(&chosen_format);

    let total_data_size = pixels_count as usize * pixels_size;

    let (tx, rx) = if target.is_none() {
        let (tx, rx) = mpsc::channel();
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    let pixel_buffer = target.as_ref().map(|buf| buf.get_id()).unwrap_or(0);

    if let Some(pixel_buffer) = target {
        assert!(pixel_buffer.get_size() >= total_data_size);
        pixel_buffer::store_infos(pixel_buffer, dimensions, chosen_format);
    }

    context.exec(move |mut ctxt| {
        unsafe {
            // binding framebuffer
            fbo::bind_framebuffer(&mut ctxt, fbo, false, true);

            // adjusting glReadBuffer
            ctxt.gl.ReadBuffer(readbuffer);

            // adjusting data alignement
            if ctxt.state.pixel_store_pack_alignment != 1 {
                ctxt.state.pixel_store_pack_alignment = 1;
                ctxt.gl.PixelStorei(gl::PACK_ALIGNMENT, 1);
            }

            // binding buffer
            if ctxt.state.pixel_pack_buffer_binding != pixel_buffer {
                ctxt.gl.BindBuffer(gl::PIXEL_PACK_BUFFER, pixel_buffer);
                ctxt.state.pixel_pack_buffer_binding = pixel_buffer;
            }

            // reading
            if pixel_buffer == 0 {
                let data_size = pixels_count as usize * pixels_size / mem::size_of::<P>();
                let mut data: Vec<P> = Vec::with_capacity(data_size);
                ctxt.gl.ReadPixels(0, 0, dimensions.0 as gl::types::GLint,
                                   dimensions.1 as gl::types::GLint, format, gltype,
                                   data.as_mut_ptr() as *mut libc::c_void);
                data.set_len(data_size);
                tx.unwrap().send(data).ok();

            } else {
                ctxt.gl.ReadPixels(0, 0, dimensions.0 as gl::types::GLint,
                                   dimensions.1 as gl::types::GLint, format, gltype,
                                   ptr::null_mut());
            }
        }
    });

    rx.map(|rx| {
        let data = texture::RawImage2d {
            data: ::std::borrow::Cow::Owned(rx.recv().unwrap()),
            width: dimensions.0 as u32,
            height: dimensions.1 as u32,
            format: chosen_format,
        };

        texture::Texture2dDataSink::from_raw(data)
    })
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
