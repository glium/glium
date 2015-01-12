use std::sync::mpsc;

use Display;

use fbo;
use texture;

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
    read_impl(fbo, atch, dimensions, &display.context.context)
}

pub fn read_from_default_fb<P, T>(attachment: gl::types::GLenum, display: &Display) -> T          // TODO: remove Clone for P
                                  where P: texture::PixelValue + Clone + Send,
                                  T: texture::Texture2dData<Data = P>
{
    let (w, h) = display.get_framebuffer_dimensions();
    let (w, h) = (w as u32, h as u32);      // TODO: remove this conversion
    read_impl(0, attachment, (w, h), &display.context.context)
}

fn read_impl<P, T>(fbo: gl::types::GLuint, readbuffer: gl::types::GLenum,
                   dimensions: (u32, u32), context: &context::Context) -> T          // TODO: remove Clone for P
                   where P: texture::PixelValue + Clone + Send,
                   T: texture::Texture2dData<Data = P>
{
    use std::mem;

    let pixels_count = dimensions.0 * dimensions.1;

    let pixels_size = texture::Texture2dData::get_format(None::<T>).get_size();
    let (format, gltype) = texture::Texture2dData::get_format(None::<T>).to_gl_enum();

    let (tx, rx) = mpsc::channel();
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

            // reading
            let total_data_size = pixels_count  as usize * pixels_size / mem::size_of::<P>();
            let mut data: Vec<P> = Vec::with_capacity(total_data_size);
            ctxt.gl.ReadPixels(0, 0, dimensions.0 as gl::types::GLint,
                               dimensions.1 as gl::types::GLint, format, gltype,
                               data.as_mut_ptr() as *mut libc::c_void);
            data.set_len(total_data_size);
            tx.send(data).ok();
        }
    });

    let data = rx.recv().unwrap();
    texture::Texture2dData::from_vec(data, dimensions.0 as u32)
}
