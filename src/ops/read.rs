use std::ptr;
use std::sync::mpsc;

use Display;
use pixel_buffer::{self, PixelBuffer};

use fbo;
use texture;

use GlObject;
use libc;
use context;
use gl;

pub fn read_attachment<P, T>(attachment: &fbo::Attachment, dimensions: (u32, u32),
                             display: &Display) -> T          // TODO: remove Clone for P
                             where P: texture::PixelValue + Clone + Send,
                             T: texture::Texture2dData<Data = P>
{
    let (fbo, atch) = display.context.framebuffer_objects.as_ref().unwrap()
                             .get_framebuffer_for_reading(attachment, &display.context.context);
    read_impl(fbo, atch, dimensions, None, &display.context.context).unwrap()
}

/// Panics if the pixel buffer is not big enough.
pub fn read_attachment_to_pb<P, T>(attachment: &fbo::Attachment, dimensions: (u32, u32),
                                   dest: &mut PixelBuffer<T>, display: &Display)          // TODO: remove Clone for P
                                   where P: texture::PixelValue + Clone + Send,
                                   T: texture::Texture2dData<Data = P>
{
    let (fbo, atch) = display.context.framebuffer_objects.as_ref().unwrap()
                             .get_framebuffer_for_reading(attachment, &display.context.context);
    read_impl(fbo, atch, dimensions, Some(dest), &display.context.context);
}

pub fn read_from_default_fb<P, T>(attachment: gl::types::GLenum, display: &Display) -> T          // TODO: remove Clone for P
                                  where P: texture::PixelValue + Clone + Send,
                                  T: texture::Texture2dData<Data = P>
{
    let (w, h) = display.get_framebuffer_dimensions();
    let (w, h) = (w as u32, h as u32);      // TODO: remove this conversion
    read_impl(0, attachment, (w, h), None, &display.context.context).unwrap()
}

/// Panics if the pixel buffer is not big enough.
pub fn read_from_default_fb_to_pb<P, T>(attachment: gl::types::GLenum,
                                        dest: &mut PixelBuffer<T>, display: &Display)          // TODO: remove Clone for P
                                        where P: texture::PixelValue + Clone + Send,
                                        T: texture::Texture2dData<Data = P>
{
    let (w, h) = display.get_framebuffer_dimensions();
    let (w, h) = (w as u32, h as u32);      // TODO: remove this conversion
    read_impl(0, attachment, (w, h), Some(dest), &display.context.context);
}

fn read_impl<P, T>(fbo: gl::types::GLuint, readbuffer: gl::types::GLenum,
                   dimensions: (u32, u32), target: Option<&mut PixelBuffer<T>>,
                   context: &context::Context) -> Option<T>          // TODO: remove Clone for P
                   where P: texture::PixelValue + Clone + Send,
                   T: texture::Texture2dData<Data = P>
{
    use std::mem;

    let pixels_count = dimensions.0 * dimensions.1;

    let pixels_size = <T as texture::Texture2dData>::get_format().get_size();
    let (format, gltype) = <T as texture::Texture2dData>::get_format().to_gl_enum();

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
        pixel_buffer::store_width(pixel_buffer, dimensions.0);
    }

    context.exec(move |: mut ctxt| {
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
        let data = rx.recv().unwrap();
        texture::Texture2dData::from_vec(data, dimensions.0 as u32)
    })
}
