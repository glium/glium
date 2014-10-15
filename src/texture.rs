use data_types;
use gl;
use libc;
use std::fmt;
use std::mem;
use std::sync::Arc;

/// Trait that describes a texture.
pub trait Texture {
    /// Returns a reference to an opaque type necessary to make things work.
    #[experimental = "May be changed to something totally different"]
    fn get_implementation(&self) -> &TextureImplementation;

    /// Returns the width in pixels of the texture.
    fn get_width(&self) -> u32 {
        self.get_implementation().width
    }

    /// Returns the height in pixels of the texture, or `None` for one dimension textures.
    fn get_height(&self) -> Option<u32> {
        self.get_implementation().height.clone()
    }
}

/// A one-dimensional texture.
pub struct Texture1D(TextureImplementation);

impl Texture1D {
    /// Creates a one-dimensional texture.
    pub fn new<T: data_types::GLDataTuple>(display: &super::Display, data: &[T]) -> Texture1D {
        Texture1D(TextureImplementation::new(display, data, data.len() as u32, None, None, None))
    }
}

impl Texture for Texture1D {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// An array of one-dimensional textures.
pub struct Texture1DArray(TextureImplementation);

impl Texture1DArray {
    /// Creates an array of one-dimensional textures.
    pub fn new<T: data_types::GLDataTuple>(display: &super::Display, data: &[T], width: u32, array_size: u32) -> Texture1DArray {
        Texture1DArray(TextureImplementation::new(display, data, width, None, None, Some(array_size)))
    }
}

impl Texture for Texture1DArray {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// A two-dimensional texture. This is usually the texture that you want to use.
pub struct Texture2D(TextureImplementation);

impl Texture2D {
    /// Creates a two-dimensional texture.
    pub fn new<T: data_types::GLDataTuple>(display: &super::Display, data: &[T], width: u32,
                                           height: u32) -> Texture2D
    {
        Texture2D(TextureImplementation::new(display, data, width, Some(height), None, None))
    }
}

impl Texture for Texture2D {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// An array of two-dimensional textures.
pub struct Texture2DArray(TextureImplementation);

impl Texture2DArray {
    /// Creates an array of two-dimensional textures.
    pub fn new<T: data_types::GLDataTuple>(display: &super::Display, data: &[T], width: u32, height: u32, array_size: u32) -> Texture2DArray {
        Texture2DArray(TextureImplementation::new(display, data, width, Some(height), None, Some(array_size)))
    }
}

impl Texture for Texture2DArray {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// A three-dimensional texture.
pub struct Texture3D(TextureImplementation);

impl Texture3D {
    /// Creates a three-dimensional texture.
    pub fn new<T: data_types::GLDataTuple>(display: &super::Display, data: &[T], width: u32,
                                           height: u32, depth: u32) -> Texture3D
    {
        Texture3D(TextureImplementation::new(display, data, width, Some(height), Some(depth), None))
    }
}

impl Texture for Texture3D {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// Opaque type that is used to make things work.
pub struct TextureImplementation {
    display: Arc<super::DisplayImpl>,
    id: gl::types::GLuint,
    bind_point: gl::types::GLenum,
    width: u32,
    height: Option<u32>,
    depth: Option<u32>,
    array_size: Option<u32>,
}

/// This function is not visible outside of `glium`.
#[doc(hidden)]
pub fn get_id(texture: &TextureImplementation) -> gl::types::GLuint {
    texture.id
}

impl TextureImplementation {
    /// Builds a new texture.
    fn new<T: data_types::GLDataTuple>(display: &super::Display, data: &[T], width: u32,
        height: Option<u32>, depth: Option<u32>, array_size: Option<u32>) -> TextureImplementation
    {
        let element_components = data_types::GLDataTuple::get_num_elems(None::<T>);

        if width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint * array_size.unwrap_or(1) as uint != data.len() {
            fail!("Texture data has different size from width*height*depth*array_size*elemLen");
        }

        let texture_type = if height.is_none() && depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }
        } else if depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_2D } else { gl::TEXTURE_2D_ARRAY }
        } else {
            gl::TEXTURE_3D
        };

        let data_type = data_types::GLDataTuple::get_gl_type(None::<T>);
        let data_raw: *const libc::c_void = unsafe { mem::transmute(data.as_ptr()) };

        let (internal_data_format, data_format, data_type) = match (element_components, data_type) {
            (1, gl::BYTE)           => (gl::RED, gl::RED, gl::BYTE),
            (1, gl::UNSIGNED_BYTE)  => (gl::RED, gl::RED, gl::UNSIGNED_BYTE),
            (1, gl::SHORT)          => (gl::RED, gl::RED, gl::SHORT),
            (1, gl::UNSIGNED_SHORT) => (gl::RED, gl::RED, gl::UNSIGNED_SHORT),
            (1, gl::INT)            => (gl::RED, gl::RED, gl::INT),
            (1, gl::UNSIGNED_INT)   => (gl::RED, gl::RED, gl::UNSIGNED_INT),
            (1, gl::FLOAT)          => (gl::R32F, gl::RED, gl::FLOAT),

            (2, gl::BYTE)           => (gl::RG, gl::RG, gl::BYTE),
            (2, gl::UNSIGNED_BYTE)  => (gl::RG, gl::RG, gl::UNSIGNED_BYTE),
            (2, gl::SHORT)          => (gl::RG, gl::RG, gl::SHORT),
            (2, gl::UNSIGNED_SHORT) => (gl::RG, gl::RG, gl::UNSIGNED_SHORT),
            (2, gl::INT)            => (gl::RG, gl::RG, gl::INT),
            (2, gl::UNSIGNED_INT)   => (gl::RG, gl::RG, gl::UNSIGNED_INT),
            (2, gl::FLOAT)          => (gl::RG32F, gl::RG, gl::FLOAT),

            (3, gl::BYTE)           => (gl::RGB, gl::RGB, gl::BYTE),
            (3, gl::UNSIGNED_BYTE)  => (gl::RGB, gl::RGB, gl::UNSIGNED_BYTE),
            (3, gl::SHORT)          => (gl::RGB, gl::RGB, gl::SHORT),
            (3, gl::UNSIGNED_SHORT) => (gl::RGB, gl::RGB, gl::UNSIGNED_SHORT),
            (3, gl::INT)            => (gl::RGB, gl::RGB, gl::INT),
            (3, gl::UNSIGNED_INT)   => (gl::RGB, gl::RGB, gl::UNSIGNED_INT),
            (3, gl::FLOAT)          => (gl::RGB32F, gl::RGB, gl::FLOAT),

            (4, gl::BYTE)           => (gl::RGBA, gl::RGBA, gl::BYTE),
            (4, gl::UNSIGNED_BYTE)  => (gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE),
            (4, gl::SHORT)          => (gl::RGBA, gl::RGBA, gl::SHORT),
            (4, gl::UNSIGNED_SHORT) => (gl::RGBA, gl::RGBA, gl::UNSIGNED_SHORT),
            (4, gl::INT)            => (gl::RGBA, gl::RGBA, gl::INT),
            (4, gl::UNSIGNED_INT)   => (gl::RGBA, gl::RGBA, gl::UNSIGNED_INT),
            (4, gl::FLOAT)          => (gl::RGBA32F, gl::RGBA, gl::FLOAT),

            _ => fail!("unsupported texture type")
        };

        let (tx, rx) = channel();
        display.context.context.exec(proc(gl, _state) {
            unsafe {
                gl.PixelStorei(gl::UNPACK_ALIGNMENT, if width % 4 == 0 { 4 } else if height.unwrap_or(1) % 2 == 0 { 2 } else { 1 });

                let id: gl::types::GLuint = mem::uninitialized();
                gl.GenTextures(1, mem::transmute(&id));

                gl.BindTexture(texture_type, id);

                gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                if height.is_some() || depth.is_some() || array_size.is_some() {
                    gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                }
                if depth.is_some() || array_size.is_some() {
                    gl.TexParameteri(texture_type, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
                }
                gl.TexParameteri(texture_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);

                if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
                    gl.TexImage3D(texture_type, 0, internal_data_format as i32, width as i32, height.unwrap() as i32, if let Some(d) = depth { d } else { array_size.unwrap_or(1) } as i32, 0, data_format as u32, data_type, data_raw);
                } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                    gl.TexImage2D(texture_type, 0, internal_data_format as i32, width as i32, height.unwrap() as i32, 0, data_format as u32, data_type, data_raw);
                } else {
                    gl.TexImage1D(texture_type, 0, internal_data_format as i32, width as i32, 0, data_format as u32, data_type, data_raw);
                }

                if gl.GenerateMipmap.is_loaded() {
                    gl.GenerateMipmap(texture_type);
                } else if gl.GenerateMipmapEXT.is_loaded() {
                    gl.GenerateMipmapEXT(texture_type);
                }

                tx.send(id);
            }
        });

        TextureImplementation {
            display: display.context.clone(),
            id: rx.recv(),
            bind_point: texture_type,
            width: width,
            height: height,
            depth: depth,
            array_size: array_size,
        }
    }

    /// Start drawing on this texture.
    fn draw(&mut self) -> super::Target {
        let display = self.display.clone();
        let fbo = super::FrameBufferObject::new(display.clone());

        // binding the texture to the FBO
        {
            let my_id = self.id.clone();
            let fbo_id = fbo.id;
            self.display.context.exec(proc(gl, _state) {
                gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_id);
                gl.FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, my_id, 0);
            });
        }

        // returning the target
        super::Target {
            display: display,
            display_hold: None,
            dimensions: (self.width as uint, self.height.unwrap_or(1) as uint),
            texture: Some(self),
            framebuffer: Some(fbo),
            execute_end: None,
        }
    }

    /// Reads the content of the texture.
    ///
    /// Same as `read_mipmap` with `level` as `0`.
    // TODO: draft ; must be checked and turned public
    fn read(&self) -> Vec<u8> {
        self.read_mipmap(0)
    }

    /// Reads the content of one of the mipmaps the texture.
    ///
    /// Returns a 2D array of pixels.
    /// Each pixel has R, G and B components between 0 and 255.
    // TODO: draft ; must be checked and turned public
    fn read_mipmap(&self, level: uint) -> Vec<u8> {
        unimplemented!()
        /*let bind_point = self.bind_point;
        let id = self.id;
        let buffer_size = self.width * self.height * self.depth *
            self.array_size * 3;

        if level != 0 {
            unimplemented!()
        }

        self.display.context.exec(proc(gl, _state) {
            let mut buffer = Vec::from_elem(buffer_size, 0u8);

            unsafe {
                gl.BindTexture(bind_point, id);
                gl.GetTexImage(bind_point, 0 as gl::types::GLint, gl::RGBA_INTEGER, gl::UNSIGNED_BYTE,
                    buffer.as_mut_ptr() as *mut libc::c_void);
            }

            buffer
        }).get()*/
    }
}

impl fmt::Show for TextureImplementation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        (format!("Texture #{} (dimensions: {}x{}x{})", self.id,
            self.width, self.height, self.depth)).fmt(formatter)
    }
}

impl Drop for TextureImplementation {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(proc(gl, _state) {
            unsafe { gl.DeleteTextures(1, [ id ].as_ptr()); }
        });
    }
}
