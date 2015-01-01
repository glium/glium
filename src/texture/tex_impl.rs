use Display;

use super::PixelValue;

use gl;
use GlObject;
use context::GlVersion;

use libc;
use std::fmt;
use std::mem;
use std::ptr;

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
            if width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint != data.len() &&
               width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint * 2 != data.len() &&
               width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint * 3 != data.len() &&
               width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
                array_size.unwrap_or(1) as uint * 4 != data.len()
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

                ctxt.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);

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

                tx.send(id);
            }
        });

        TextureImplementation {
            display: display.clone(),
            id: rx.recv(),
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
    #[cfg(feature = "gl_extensions")]
    pub fn read<P>(&self, level: u32) -> Vec<P> where P: PixelValue {
        assert_eq!(level, 0);   // TODO: 

        let pixels_count = (self.width * self.height.unwrap_or(1) * self.depth.unwrap_or(1))
                            as uint;

        // FIXME: WRONG
        let (format, gltype) = PixelValue::get_format(None::<P>).to_gl_enum();
        let my_id = self.id;

        let (tx, rx) = channel();
        self.display.context.context.exec(move |: ctxt| {
            unsafe {
                let mut data: Vec<P> = Vec::with_capacity(pixels_count);

                ctxt.gl.PixelStorei(gl::PACK_ALIGNMENT, 1);

                if ctxt.version >= &GlVersion(4, 5) {
                    ctxt.gl.GetTextureImage(my_id, level as gl::types::GLint, format, gltype,
                        (pixels_count * mem::size_of::<P>()) as gl::types::GLsizei,
                        data.as_mut_ptr() as *mut libc::c_void);

                } else if ctxt.extensions.gl_ext_direct_state_access {
                    ctxt.gl.GetTextureImageEXT(my_id, gl::TEXTURE_2D, level as gl::types::GLint,
                        format, gltype, data.as_mut_ptr() as *mut libc::c_void);

                } else {
                    ctxt.gl.BindTexture(gl::TEXTURE_2D, my_id);
                    ctxt.gl.GetTexImage(gl::TEXTURE_2D, level as gl::types::GLint, format, gltype,
                        data.as_mut_ptr() as *mut libc::c_void);
                }

                data.set_len(pixels_count);
                tx.send(data);
            }
        });

        rx.recv()
    }

    /// Returns the `Display` associated to this texture.
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

    /// Returns the size of array of the texture.
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
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        (format!("Texture #{} (dimensions: {}x{}x{})", self.id,
            self.width, self.height, self.depth)).fmt(formatter)
    }
}

impl Drop for TextureImplementation {
    fn drop(&mut self) {
        use fbo;

        // removing FBOs which contain this texture
        {
            let mut fbos = self.display.context.framebuffer_objects.lock().unwrap();

            let to_delete = fbos.keys().filter(|b| {
                b.colors.iter().find(|&&(_, id)| id == fbo::Attachment::Texture(self.id)).is_some() ||
                b.depth == Some(fbo::Attachment::Texture(self.id)) || b.stencil == Some(fbo::Attachment::Texture(self.id))
            }).map(|k| k.clone()).collect::<Vec<_>>();

            for k in to_delete.into_iter() {
                fbos.remove(&k);
            }
        }

        let id = self.id.clone();
        self.display.context.context.exec(move |: ctxt| {
            unsafe { ctxt.gl.DeleteTextures(1, [ id ].as_ptr()); }
        });
    }
}
