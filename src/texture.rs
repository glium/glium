/*!

A texture is an image available for drawing.

To create a texture, you must first create a struct that implements one of `Texture1DData`,
 `Texture2DData` or `Texture3DData`. Then call the appropriate `new` function of the type of
 texture that you desire.

The most common type of texture is a `Texture2D` (the two dimensions being the width and height),
 it is what you will use most of the time.

**Note**: `TextureCube` does not yet exist.

*/

use context::GlVersion;
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

    /// Returns the depth in pixels of the texture, or `None` for one or two dimension textures.
    fn get_depth(&self) -> Option<u32> {
        self.get_implementation().depth.clone()
    }

    /// Returns the number of textures in the array, or `None` for non-arrays.
    fn get_array_size(&self) -> Option<u32> {
        self.get_implementation().array_size.clone()
    }
}

/// A trait that must be implemented for any type that can represent the value of a pixel.
#[experimental = "Will be rewritten after UFCS land"]
pub trait PixelValue: Copy + Send {     // TODO: Clone, but [T, ..N] doesn't impl Clone
    /// Returns the `GLenum` corresponding to the type of this pixel.
    fn get_gl_type(_: Option<Self>) -> gl::types::GLenum;
    /// Returns the number of color components.
    fn get_num_elems(_: Option<Self>) -> gl::types::GLint;
}

// TODO: hacky
impl PixelValue for i8 {
    fn get_gl_type(_: Option<(i8)>) -> gl::types::GLenum { gl::BYTE }
    fn get_num_elems(_: Option<(i8)>) -> gl::types::GLint { 1 }
}

impl PixelValue for u8 {
    fn get_gl_type(_: Option<(u8)>) -> gl::types::GLenum { gl::UNSIGNED_BYTE }
    fn get_num_elems(_: Option<(u8)>) -> gl::types::GLint { 1 }
}

impl PixelValue for f32 {
    fn get_gl_type(_: Option<(f32)>) -> gl::types::GLenum { gl::FLOAT }
    fn get_num_elems(_: Option<(f32)>) -> gl::types::GLint { 1 }
}

impl<T: PixelValue> PixelValue for (T, T) {
    fn get_gl_type(_: Option<(T, T)>) -> gl::types::GLenum { PixelValue::get_gl_type(None::<T>) }
    fn get_num_elems(_: Option<(T, T)>) -> gl::types::GLint { PixelValue::get_num_elems(None::<T>) * 2 }
}

impl<T: PixelValue> PixelValue for (T, T, T) {
    fn get_gl_type(_: Option<(T, T, T)>) -> gl::types::GLenum { PixelValue::get_gl_type(None::<T>) }
    fn get_num_elems(_: Option<(T, T, T)>) -> gl::types::GLint { PixelValue::get_num_elems(None::<T>) * 3 }
}

impl<T: PixelValue> PixelValue for (T, T, T, T) {
    fn get_gl_type(_: Option<(T, T, T, T)>) -> gl::types::GLenum { PixelValue::get_gl_type(None::<T>) }
    fn get_num_elems(_: Option<(T, T, T, T)>) -> gl::types::GLint { PixelValue::get_num_elems(None::<T>) * 4 }
}

impl<T: PixelValue> PixelValue for [T, ..2] {
    fn get_gl_type(_: Option<[T, ..2]>) -> gl::types::GLenum { PixelValue::get_gl_type(None::<T>) }
    fn get_num_elems(_: Option<[T, ..2]>) -> gl::types::GLint { PixelValue::get_num_elems(None::<T>) * 2 }
}

impl<T: PixelValue> PixelValue for [T, ..3] {
    fn get_gl_type(_: Option<[T, ..3]>) -> gl::types::GLenum { PixelValue::get_gl_type(None::<T>) }
    fn get_num_elems(_: Option<[T, ..3]>) -> gl::types::GLint { PixelValue::get_num_elems(None::<T>) * 3 }
}

impl<T: PixelValue> PixelValue for [T, ..4] {
    fn get_gl_type(_: Option<[T, ..4]>) -> gl::types::GLenum { PixelValue::get_gl_type(None::<T>) }
    fn get_num_elems(_: Option<[T, ..4]>) -> gl::types::GLint { PixelValue::get_num_elems(None::<T>) * 4 }
}


/// A one-dimensional texture.
pub struct Texture1D(TextureImplementation);

impl Texture1D {
    /// Creates a one-dimensional texture.
    pub fn new<P: PixelValue, T: Texture1DData<P>>(display: &super::Display, data: T) -> Texture1D {
        let data = data.into_vec();
        let width = data.len() as u32;
        Texture1D(TextureImplementation::new(display, data, width, None, None, None))
    }
}

impl Texture for Texture1D {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// Trait that describes data for a one-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture1DData<P> {
    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;
}

impl<P: PixelValue> Texture1DData<P> for Vec<P> {
    fn into_vec(self) -> Vec<P> {
        self
    }
}

impl<'a, P: PixelValue + Clone> Texture1DData<P> for &'a [P] {
    fn into_vec(self) -> Vec<P> {
        self.to_vec()
    }
}

/// An array of one-dimensional textures.
pub struct Texture1DArray(TextureImplementation);

impl Texture1DArray {
    /// Creates an array of one-dimensional textures.
    ///
    /// # Panic
    ///
    /// Panics if all the elements don't have the same dimensions.
    pub fn new<P: PixelValue, T: Texture1DData<P>>(display: &super::Display, data: Vec<T>)
        -> Texture1DArray
    {
        let array_size = data.len();
        let mut width = 0;
        let data = data.into_iter().flat_map(|t| {
            let d = t.into_vec(); width = d.len(); d.into_iter()
        }).collect();

        Texture1DArray(TextureImplementation::new(display, data, width as u32, None, None,
            Some(array_size as u32)))
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
    pub fn new<P: PixelValue, T: Texture2DData<P>>(display: &super::Display, data: T) -> Texture2D {
        let dimensions = data.get_dimensions();
        let data = data.into_vec();

        Texture2D(TextureImplementation::new(display, data, dimensions.0, Some(dimensions.1),
            None, None))
    }

    /// Starts drawing on the texture.
    ///
    /// This does not erase the existing content of the texture as long as you don't call
    ///  `clear_colors` on the `Target`.
    pub fn draw(&mut self) -> super::Target {
        self.0.draw()
    }
}

impl Texture for Texture2D {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

impl ::blit::BlitSurface for Texture2D {
    unsafe fn get_implementation(&self) -> ::BlitSurfaceImpl {
        let fbo = self.0.build_fbo();
        let id = fbo.id;

        ::BlitSurfaceImpl {
            display: self.0.display.clone(),
            fbo_storage: Some(fbo),
            fbo: Some(id),
        }
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.0.width, self.0.height.unwrap_or(1))
    }
}

/// Trait that describes data for a two-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture2DData<P> {
    /// Returns the dimensions of the texture.
    fn get_dimensions(&self) -> (u32, u32);

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;

    /// Builds a new object from raw data.
    fn from_vec(Vec<P>, width: u32) -> Self;
}

impl<P: PixelValue + Clone> Texture2DData<P> for Vec<Vec<P>> {      // TODO: remove Clone
    fn get_dimensions(&self) -> (u32, u32) {
        (self.iter().next().map(|e| e.len()).unwrap_or(0) as u32, self.len() as u32)
    }

    fn into_vec(self) -> Vec<P> {
        self.into_iter().flat_map(|e| e.into_iter()).collect()
    }

    fn from_vec(data: Vec<P>, width: u32) -> Vec<Vec<P>> {
        data.as_slice().chunks(width as uint).map(|e| e.to_vec()).collect()
    }
}

/// An array of two-dimensional textures.
pub struct Texture2DArray(TextureImplementation);

impl Texture2DArray {
    /// Creates an array of two-dimensional textures.
    ///
    /// # Panic
    ///
    /// Panics if all the elements don't have the same dimensions.
    pub fn new<P: PixelValue, T: Texture2DData<P>>(display: &super::Display, data: Vec<T>)
        -> Texture2DArray
    {
        let array_size = data.len();
        let mut dimensions = (0, 0);
        let data = data.into_iter().flat_map(|t| {
            dimensions = t.get_dimensions(); t.into_vec().into_iter()
        }).collect();

        Texture2DArray(TextureImplementation::new(display, data, dimensions.0, Some(dimensions.1),
            None, Some(array_size as u32)))
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
    pub fn new<P: PixelValue, T: Texture3DData<P>>(display: &super::Display, data: T) -> Texture3D {
        let dimensions = data.get_dimensions();
        let data = data.into_vec();
        Texture3D(TextureImplementation::new(display, data, dimensions.0, Some(dimensions.1),
            Some(dimensions.2), None))
    }
}

impl Texture for Texture3D {
    fn get_implementation(&self) -> &TextureImplementation {
        &self.0
    }
}

/// Trait that describes data for a three-dimensional texture.
#[experimental = "Will be rewritten to use an associated type"]
pub trait Texture3DData<P> {
    /// Returns the dimensions of the texture.
    fn get_dimensions(&self) -> (u32, u32, u32);

    /// Returns a vec where each element is a pixel of the texture.
    fn into_vec(self) -> Vec<P>;
}

impl<P: PixelValue> Texture3DData<P> for Vec<Vec<Vec<P>>> {
    fn get_dimensions(&self) -> (u32, u32, u32) {
        (self.iter().next().and_then(|e| e.iter().next()).map(|e| e.len()).unwrap_or(0) as u32,
            self.iter().next().map(|e| e.len()).unwrap_or(0) as u32, self.len() as u32)
    }

    fn into_vec(self) -> Vec<P> {
        self.into_iter().flat_map(|e| e.into_iter()).flat_map(|e| e.into_iter()).collect()
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
    fn new<P: PixelValue>(display: &super::Display, data: Vec<P>, width: u32,
        height: Option<u32>, depth: Option<u32>, array_size: Option<u32>) -> TextureImplementation
    {
        let element_components = PixelValue::get_num_elems(None::<P>);

        if width as uint * height.unwrap_or(1) as uint * depth.unwrap_or(1) as uint *
            array_size.unwrap_or(1) as uint != data.len()
        {
            panic!("Texture data has different size from width*height*depth*array_size*elemLen");
        }

        let texture_type = if height.is_none() && depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }
        } else if depth.is_none() {
            if array_size.is_none() { gl::TEXTURE_2D } else { gl::TEXTURE_2D_ARRAY }
        } else {
            gl::TEXTURE_3D
        };

        let data_type = PixelValue::get_gl_type(None::<P>);

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

            _ => panic!("unsupported texture type")
        };

        let (tx, rx) = channel();
        display.context.context.exec(proc(gl, _state, version, _) {
            unsafe {
                let data = data;
                let data_raw: *const libc::c_void = mem::transmute(data.as_slice().as_ptr());

                gl.PixelStorei(gl::UNPACK_ALIGNMENT, if width % 4 == 0 {
                    4
                } else if height.unwrap_or(1) % 2 == 0 {
                    2
                } else {
                    1
                });

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
                gl.TexParameteri(texture_type, gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as i32);

                if texture_type == gl::TEXTURE_3D || texture_type == gl::TEXTURE_2D_ARRAY {
                    gl.TexImage3D(texture_type, 0, internal_data_format as i32, width as i32,
                        height.unwrap() as i32,
                        if let Some(d) = depth { d } else { array_size.unwrap_or(1) } as i32, 0,
                        data_format as u32, data_type, data_raw);

                } else if texture_type == gl::TEXTURE_2D || texture_type == gl::TEXTURE_1D_ARRAY {
                    gl.TexImage2D(texture_type, 0, internal_data_format as i32, width as i32,
                        height.unwrap() as i32, 0, data_format as u32, data_type, data_raw);
                } else {
                    gl.TexImage1D(texture_type, 0, internal_data_format as i32, width as i32, 0,
                        data_format as u32, data_type, data_raw);
                }

                if version >= &GlVersion(3, 0) {
                    gl.GenerateMipmap(texture_type);
                } else {
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
        use std::kinds::marker::ContravariantLifetime;

        let display = self.display.clone();
        let fbo = self.build_fbo();

        // returning the target
        super::Target {
            display: display,
            marker: ContravariantLifetime,
            dimensions: (self.width as uint, self.height.unwrap_or(1) as uint),
            framebuffer: Some(fbo),
            execute_end: None,
        }
    }

    /// Builds a framebuffer object to draw on this texture.
    fn build_fbo(&self) -> ::FrameBufferObject {
        let display = self.display.clone();
        let fbo = super::FrameBufferObject::new(display);

        // binding the texture to the FBO
        {
            let my_id = self.id.clone();
            let fbo_id = fbo.id;
            self.display.context.exec(proc(gl, state, version, extensions) {
                if version >= &GlVersion(4, 5) {
                    gl.NamedFramebufferTexture(fbo_id, gl::COLOR_ATTACHMENT0, my_id, 0);

                } else if extensions.gl_ext_direct_state_access &&
                          extensions.gl_ext_geometry_shader4
                {
                    gl.NamedFramebufferTextureEXT(fbo_id, gl::COLOR_ATTACHMENT0, my_id, 0);

                } else if version >= &GlVersion(3, 2) {
                    gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo_id);
                    state.draw_framebuffer = Some(fbo_id);
                    gl.FramebufferTexture(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, my_id, 0);

                } else if version >= &GlVersion(3, 0) {
                    gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo_id);
                    state.draw_framebuffer = Some(fbo_id);
                    gl.FramebufferTexture2D(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D, my_id, 0);

                } else {
                    gl.BindFramebufferEXT(gl::FRAMEBUFFER_EXT, fbo_id);
                    state.draw_framebuffer = Some(fbo_id);
                    state.read_framebuffer = Some(fbo_id);
                    gl.FramebufferTexture2DEXT(gl::FRAMEBUFFER_EXT, gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D, my_id, 0);
                }
            });
        }

        fbo
    }

    /// Reads the content of the texture.
    ///
    /// Same as `read_mipmap` with `level` as `0`.
    // TODO: draft ; must be checked and turned public
    #[allow(dead_code)]     // remove
    fn read(&self) -> Vec<u8> {
        self.read_mipmap(0)
    }

    /// Reads the content of one of the mipmaps the texture.
    ///
    /// Returns a 2D array of pixels.
    /// Each pixel has R, G and B components between 0 and 255.
    // TODO: draft ; must be checked and turned public
    #[allow(dead_code)]     // remove
    fn read_mipmap(&self, _level: uint) -> Vec<u8> {
        unimplemented!()
        /*let bind_point = self.bind_point;
        let id = self.id;
        let buffer_size = self.width * self.height * self.depth *
            self.array_size * 3;

        if level != 0 {
            unimplemented!()
        }

        self.display.context.exec(proc(gl, _state, _, _) {
            let mut buffer = Vec::from_elem(buffer_size, 0u8);

            unsafe {
                gl.BindTexture(bind_point, id);
                gl.GetTexImage(bind_point, 0 as gl::types::GLint, gl::RGBA_INTEGER,
                    gl::UNSIGNED_BYTE, buffer.as_mut_ptr() as *mut libc::c_void);
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
        self.display.context.exec(proc(gl, _state, _, _) {
            unsafe { gl.DeleteTextures(1, [ id ].as_ptr()); }
        });
    }
}
