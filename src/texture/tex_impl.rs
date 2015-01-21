use Display;

use gl;
use GlObject;
use context::GlVersion;

use pixel_buffer::PixelBuffer;
use texture::{Texture2dData, PixelValue};

use libc;
use std::fmt;
use std::mem;
use std::ptr;
use std::sync::mpsc::channel;

use ops;
use fbo;

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
    pub fn new<P>(display: &Display, format: gl::types::GLenum, data: Option<Vec<P>>,
        client_format: gl::types::GLenum, client_type: gl::types::GLenum, width: u32,
        height: Option<u32>, depth: Option<u32>, array_size: Option<u32>) -> TextureImplementation
        where P: Send
    {
        use std::num::Float;

        if let Some(ref data) = data {
            if width as usize * height.unwrap_or(1) as usize * depth.unwrap_or(1) as usize *
                array_size.unwrap_or(1) as usize != data.len() &&
               width as usize * height.unwrap_or(1) as usize * depth.unwrap_or(1) as usize *
                array_size.unwrap_or(1) as usize * 2 != data.len() &&
               width as usize * height.unwrap_or(1) as usize * depth.unwrap_or(1) as usize *
                array_size.unwrap_or(1) as usize * 3 != data.len() &&
               width as usize * height.unwrap_or(1) as usize * depth.unwrap_or(1) as usize *
                array_size.unwrap_or(1) as usize * 4 != data.len()
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

        let texture_levels = 1 + (::std::cmp::max(width, ::std::cmp::max(height.unwrap_or(1),
                                 depth.unwrap_or(1))) as f32).log2() as gl::types::GLsizei;

        let (tx, rx) = channel();
        display.context.context.exec(move |: ctxt| {
            unsafe {
                let data = data;
                let data_raw = if let Some(ref data) = data {
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
                    if ctxt.version >= &GlVersion(4, 2) || ctxt.extensions.gl_arb_texture_storage {
                        ctxt.gl.TexStorage3D(texture_type, texture_levels,
                                             format as gl::types::GLenum,
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
                        ctxt.gl.TexImage3D(texture_type, 0, format as i32, width as i32,
                                           height.unwrap() as i32,
                                           if let Some(d) = depth { d } else {
                                               array_size.unwrap_or(1)
                                           } as i32, 0,
                                           client_format as u32, client_type, data_raw);
                    }

                } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                    if ctxt.version >= &GlVersion(4, 2) || ctxt.extensions.gl_arb_texture_storage {
                        ctxt.gl.TexStorage2D(texture_type, texture_levels,
                                             format as gl::types::GLenum,
                                             width as gl::types::GLsizei,
                                             height.unwrap() as gl::types::GLsizei);

                        if !data_raw.is_null() {
                            ctxt.gl.TexSubImage2D(texture_type, 0, 0, 0, width as gl::types::GLsizei,
                                                  height.unwrap() as gl::types::GLsizei,
                                                  client_format, client_type, data_raw);
                        }

                    } else {
                        ctxt.gl.TexImage2D(texture_type, 0, format as i32, width as i32,
                                           height.unwrap() as i32, 0, client_format as u32,
                                           client_type, data_raw);
                    }

                } else {
                    if ctxt.version >= &GlVersion(4, 2) || ctxt.extensions.gl_arb_texture_storage {
                        ctxt.gl.TexStorage1D(texture_type, texture_levels,
                                             format as gl::types::GLenum,
                                             width as gl::types::GLsizei);

                        if !data_raw.is_null() {
                            ctxt.gl.TexSubImage1D(texture_type, 0, 0, width as gl::types::GLsizei,
                                                  client_format, client_type, data_raw);
                        }

                    } else {
                        ctxt.gl.TexImage1D(texture_type, 0, format as i32, width as i32, 0,
                                           client_format as u32, client_type, data_raw);
                    }
                }

                if ctxt.version >= &GlVersion(3, 0) {
                    ctxt.gl.GenerateMipmap(texture_type);
                } else {
                    ctxt.gl.GenerateMipmapEXT(texture_type);
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
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl fmt::Show for TextureImplementation {
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
