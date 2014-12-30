/*!
Contains everything related to vertex buffers.

The main struct is the `VertexBuffer`, which represents a buffer in the video memory
containing a list of vertices.

In order to create a vertex buffer, you must first create a struct that represents each vertex
and implement the `glium::vertex_buffer::Vertex` trait on it. The `#[vertex_format]` attribute
coming from `glium_macros` helps you doing that.

```
# #![feature(phase)]
# #[phase(plugin)]
# extern crate glium_macros;
# extern crate glium;
# extern crate glutin;
# fn main() {
#[vertex_format]
#[deriving(Copy)]
struct Vertex {
    position: [f32, ..3],
    texcoords: [f32, ..2],
}
# }
```

Then you must build a `Vec` of the vertices that you want to upload, and pass it to
`VertexBuffer::new`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# #[deriving(Copy)]
# struct Vertex {
#     position: [f32, ..3],
#     texcoords: [f32, ..2],
# }
# impl glium::vertex_buffer::Vertex for Vertex {
#     fn build_bindings(_: Option<Vertex>) -> glium::vertex_buffer::VertexFormat {
#         unimplemented!() }
# }
let data = vec![
    Vertex {
        position: [0.0, 0.0, 0.4],
        texcoords: [0.0, 1.0]
    },
    Vertex {
        position: [12.0, 4.5, -1.8],
        texcoords: [1.0, 0.5]
    },
    Vertex {
        position: [-7.124, 0.1, 0.0],
        texcoords: [0.0, 0.4]
    },
];

let vertex_buffer = glium::vertex_buffer::VertexBuffer::new(&display, data);
```

*/
use buffer::{mod, Buffer};
use gl;
use GlObject;

/// Describes the source to use for the vertices when drawing.
#[deriving(Clone, Copy)]
pub enum VerticesSource<'a> {
    /// A buffer uploaded in the video memory.
    VertexBuffer(&'a VertexBufferAny),
}

/// Objects that can be used as vertex sources.
pub trait IntoVerticesSource<'a> {
    /// Builds the `VerticesSource`.
    fn into_vertices_source(self) -> VerticesSource<'a>;
}

impl<'a> IntoVerticesSource<'a> for VerticesSource<'a> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        self
    }
}

/// A list of vertices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct VertexBuffer<T> {
    buffer: VertexBufferAny,
}

impl<T: Vertex + 'static + Send> VertexBuffer<T> {
    /// Builds a new vertex buffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #![feature(phase)]
    /// # #[phase(plugin)]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// #[vertex_format]
    /// #[deriving(Copy)]
    /// struct Vertex {
    ///     position: [f32, ..3],
    ///     texcoords: [f32, ..2],
    /// }
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let vertex_buffer = glium::VertexBuffer::new(&display, vec![
    ///     Vertex { position: [0.0,  0.0, 0.0], texcoords: [0.0, 1.0] },
    ///     Vertex { position: [5.0, -3.0, 2.0], texcoords: [1.0, 0.0] },
    /// ]);
    /// # }
    /// ```
    /// 
    pub fn new(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = Vertex::build_bindings(None::<T>);

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::STATIC_DRAW);
        let elements_size = buffer.get_elements_size();

        VertexBuffer {
            buffer: VertexBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            }
        }
    }

    /// Builds a new vertex buffer.
    ///
    /// This function will create a buffer that has better performances when it is modified
    ///  frequently.
    pub fn new_dynamic(display: &super::Display, data: Vec<T>) -> VertexBuffer<T> {
        let bindings = Vertex::build_bindings(None::<T>);

        let buffer = Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::DYNAMIC_DRAW);
        let elements_size = buffer.get_elements_size();

        VertexBuffer {
            buffer: VertexBufferAny {
                buffer: buffer,
                bindings: bindings,
                elements_size: elements_size,
            }
        }
    }
}

impl<T: Send + Copy> VertexBuffer<T> {
    /// Builds a new vertex buffer from an undeterminate data type and bindings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #![feature(phase)]
    /// # #[phase(plugin)]
    /// # extern crate glium_macros;
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// let bindings = vec![(
    ///         "position".to_string(), 0,
    ///         glium::vertex_buffer::AttributeType::F32F32,
    ///     ), (
    ///         "color".to_string(), 2 * ::std::mem::size_of::<f32>(),
    ///         glium::vertex_buffer::AttributeType::F32,
    ///     ),
    /// ];
    ///
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let data = vec![
    ///     1.0, -0.3, 409.0,
    ///     -0.4, 2.8, 715.0f32
    /// ];
    ///
    /// let vertex_buffer = unsafe {
    ///     glium::VertexBuffer::new_raw(&display, data, bindings, 3 * ::std::mem::size_of::<f32>())
    /// };
    /// # }
    /// ```
    ///
    #[experimental]
    pub unsafe fn new_raw(display: &super::Display, data: Vec<T>,
                          bindings: VertexFormat, elements_size: uint) -> VertexBuffer<T>
    {
        VertexBuffer {
            buffer: VertexBufferAny {
                buffer: Buffer::new::<buffer::ArrayBuffer, T>(display, data, gl::STATIC_DRAW),
                bindings: bindings,
                elements_size: elements_size,
            }
        }
    }

    /// Maps the buffer to allow write access to it.
    ///
    /// **Warning**: using this function can slow things down a lot because the function
    /// waits for all the previous commands to be executed before returning.
    ///
    /// # Panic
    ///
    /// OpenGL ES doesn't support mapping buffers. Using this function will thus panic.
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature.
    ///
    /// # Features
    ///
    /// Only available if the `gl_extensions` feature is enabled.
    pub fn map<'a>(&'a mut self) -> Mapping<'a, T> {
        let len = self.buffer.buffer.get_elements_count();
        let mapping = self.buffer.buffer.map::<buffer::ArrayBuffer, T>(0, len);
        Mapping(mapping)
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map` is better
    /// if you want to have multiple small reads.
    ///
    /// # Panic
    ///
    /// OpenGL ES doesn't support reading buffers. Using this function will thus panic.
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature.
    ///
    /// # Features
    ///
    /// Only available if the `gl_extensions` feature is enabled.
    #[cfg(feature = "gl_extensions")]
    pub fn read(&self) -> Vec<T> {
        self.buffer.buffer.read::<buffer::ArrayBuffer, T>()
    }

    /// Reads the content of the buffer.
    ///
    /// This function is usually better if are just doing one punctual read, while `map` is better
    /// if you want to have multiple small reads.
    ///
    /// The offset and size are expressed in number of elements.
    ///
    /// ## Panic
    ///
    /// Panics if `offset` or `offset + size` are greated than the size of the buffer.
    ///
    /// OpenGL ES doesn't support reading buffers. Using this function will thus panic.
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature.
    ///
    /// # Features
    ///
    /// Only available if the `gl_extensions` feature is enabled.
    #[cfg(feature = "gl_extensions")]
    pub fn read_slice(&self, offset: uint, size: uint) -> Vec<T> {
        self.buffer.buffer.read_slice::<buffer::ArrayBuffer, T>(offset, size)
    }
}

impl<T> VertexBuffer<T> {
    /// Returns the number of bytes between two consecutive elements in the buffer.
    pub fn get_elements_size(&self) -> uint {
        self.buffer.elements_size
    }

    /// Returns the associated `VertexFormat`.
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.buffer.bindings
    }

    /// Lose the type informations and turn the vertex buffer into a `VertexBufferAny`.
    pub fn into_vertex_buffer_any(self) -> VertexBufferAny {
        self.buffer
    }
}

impl<T> GlObject for VertexBuffer<T> {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<'a, T> IntoVerticesSource<'a> for &'a VertexBuffer<T> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::VertexBuffer(&self.buffer)
    }
}

/// A list of vertices loaded in the graphics card's memory.
///
/// Contrary to `VertexBuffer`, this struct doesn't know about the type of data
/// inside the buffer. Therefore you can't map or read it.
///
/// This struct is provided for convenience, so that you can have a `Vec<VertexBufferAny>`,
/// or return a `VertexBufferAny` instead of a `VertexBuffer<MyPrivateVertexType>`.
#[deriving(Show)]
pub struct VertexBufferAny {
    buffer: Buffer,
    bindings: VertexFormat,
    elements_size: uint,
}

impl VertexBufferAny {
    /// Returns the number of bytes between two consecutive elements in the buffer.
    pub fn get_elements_size(&self) -> uint {
        self.elements_size
    }

    /// Returns the associated `VertexFormat`.
    pub fn get_bindings(&self) -> &VertexFormat {
        &self.bindings
    }

    /// Turns the vertex buffer into a `VertexBuffer` without checking the type.
    pub unsafe fn into_vertex_buffer<T>(self) -> VertexBuffer<T> {
        VertexBuffer {
            buffer: self,
        }
    }
}

impl Drop for VertexBufferAny {
    fn drop(&mut self) {
        // removing VAOs which contain this vertex buffer
        let mut vaos = self.buffer.get_display().vertex_array_objects.lock();
        let to_delete = vaos.keys().filter(|&&(v, _, _)| v == self.buffer.get_id())
            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
    }
}

impl GlObject for VertexBufferAny {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl<'a> IntoVerticesSource<'a> for &'a VertexBufferAny {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::VertexBuffer(self)
    }
}

/// A mapping of a buffer.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
pub struct Mapping<'a, T>(buffer::Mapping<'a, buffer::ArrayBuffer, T>);

impl<'a, T> Deref<[T]> for Mapping<'a, T> {
    fn deref<'b>(&'b self) -> &'b [T] {
        self.0.deref()
    }
}

impl<'a, T> DerefMut<[T]> for Mapping<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut [T] {
        self.0.deref_mut()
    }
}

#[allow(missing_docs)]
#[deriving(Copy, Clone, Show, PartialEq, Eq)]
pub enum AttributeType {
    I8,
    I8I8,
    I8I8I8,
    I8I8I8I8,
    U8,
    U8U8,
    U8U8U8,
    U8U8U8U8,
    I16,
    I16I16,
    I16I16I16,
    I16I16I16I16,
    U16,
    U16U16,
    U16U16U16,
    U16U16U16U16,
    I32,
    I32I32,
    I32I32I32,
    I32I32I32I32,
    U32,
    U32U32,
    U32U32U32,
    U32U32U32U32,
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
}

/// Describes the layout of each vertex in a vertex buffer.
///
/// The first element is the name of the binding, the second element is the offset
/// from the start of each vertex to this element, and the third element is the type.
pub type VertexFormat = Vec<(String, uint, AttributeType)>;

/// Trait for structures that represent a vertex.
///
/// Instead of implementing this trait yourself, it is recommended to use the `#[vertex_format]`
/// attribute from `glium_macros` instead.
// TODO: this should be `unsafe`, but that would break the syntax extension
pub trait Vertex: Copy {
    /// Builds the `VertexFormat` representing the layout of this element.
    fn build_bindings(Option<Self>) -> VertexFormat;
}

/// Trait for types that can be used as vertex attributes.
pub unsafe trait Attribute {
    /// Get the type of data.
    fn get_type(_: Option<Self>) -> AttributeType;
}

unsafe impl Attribute for i8 {
    fn get_type(_: Option<i8>) -> AttributeType {
        AttributeType::I8
    }
}

unsafe impl Attribute for (i8, i8) {
    fn get_type(_: Option<(i8, i8)>) -> AttributeType {
        AttributeType::I8I8
    }
}

unsafe impl Attribute for [i8, ..2] {
    fn get_type(_: Option<[i8, ..2]>) -> AttributeType {
        AttributeType::I8I8
    }
}

unsafe impl Attribute for (i8, i8, i8) {
    fn get_type(_: Option<(i8, i8, i8)>) -> AttributeType {
        AttributeType::I8I8I8
    }
}

unsafe impl Attribute for [i8, ..3] {
    fn get_type(_: Option<[i8, ..3]>) -> AttributeType {
        AttributeType::I8I8I8
    }
}

unsafe impl Attribute for (i8, i8, i8, i8) {
    fn get_type(_: Option<(i8, i8, i8, i8)>) -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

unsafe impl Attribute for [i8, ..4] {
    fn get_type(_: Option<[i8, ..4]>) -> AttributeType {
        AttributeType::I8I8I8I8
    }
}

unsafe impl Attribute for u8 {
    fn get_type(_: Option<u8>) -> AttributeType {
        AttributeType::U8
    }
}

unsafe impl Attribute for (u8, u8) {
    fn get_type(_: Option<(u8, u8)>) -> AttributeType {
        AttributeType::U8U8
    }
}

unsafe impl Attribute for [u8, ..2] {
    fn get_type(_: Option<[u8, ..2]>) -> AttributeType {
        AttributeType::U8U8
    }
}

unsafe impl Attribute for (u8, u8, u8) {
    fn get_type(_: Option<(u8, u8, u8)>) -> AttributeType {
        AttributeType::U8U8U8
    }
}

unsafe impl Attribute for [u8, ..3] {
    fn get_type(_: Option<[u8, ..3]>) -> AttributeType {
        AttributeType::U8U8U8
    }
}

unsafe impl Attribute for (u8, u8, u8, u8) {
    fn get_type(_: Option<(u8, u8, u8, u8)>) -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

unsafe impl Attribute for [u8, ..4] {
    fn get_type(_: Option<[u8, ..4]>) -> AttributeType {
        AttributeType::U8U8U8U8
    }
}

unsafe impl Attribute for i16 {
    fn get_type(_: Option<i16>) -> AttributeType {
        AttributeType::I16
    }
}

unsafe impl Attribute for (i16, i16) {
    fn get_type(_: Option<(i16, i16)>) -> AttributeType {
        AttributeType::I16I16
    }
}

unsafe impl Attribute for [i16, ..2] {
    fn get_type(_: Option<[i16, ..2]>) -> AttributeType {
        AttributeType::I16I16
    }
}

unsafe impl Attribute for (i16, i16, i16) {
    fn get_type(_: Option<(i16, i16, i16)>) -> AttributeType {
        AttributeType::I16I16I16
    }
}

unsafe impl Attribute for [i16, ..3] {
    fn get_type(_: Option<[i16, ..3]>) -> AttributeType {
        AttributeType::I16I16I16
    }
}

unsafe impl Attribute for (i16, i16, i16, i16) {
    fn get_type(_: Option<(i16, i16, i16, i16)>) -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

unsafe impl Attribute for [i16, ..4] {
    fn get_type(_: Option<[i16, ..4]>) -> AttributeType {
        AttributeType::I16I16I16I16
    }
}

unsafe impl Attribute for u16 {
    fn get_type(_: Option<u16>) -> AttributeType {
        AttributeType::U16
    }
}

unsafe impl Attribute for (u16, u16) {
    fn get_type(_: Option<(u16, u16)>) -> AttributeType {
        AttributeType::U16U16
    }
}

unsafe impl Attribute for [u16, ..2] {
    fn get_type(_: Option<[u16, ..2]>) -> AttributeType {
        AttributeType::U16U16
    }
}

unsafe impl Attribute for (u16, u16, u16) {
    fn get_type(_: Option<(u16, u16, u16)>) -> AttributeType {
        AttributeType::U16U16U16
    }
}

unsafe impl Attribute for [u16, ..3] {
    fn get_type(_: Option<[u16, ..3]>) -> AttributeType {
        AttributeType::U16U16U16
    }
}

unsafe impl Attribute for (u16, u16, u16, u16) {
    fn get_type(_: Option<(u16, u16, u16, u16)>) -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

unsafe impl Attribute for [u16, ..4] {
    fn get_type(_: Option<[u16, ..4]>) -> AttributeType {
        AttributeType::U16U16U16U16
    }
}

unsafe impl Attribute for i32 {
    fn get_type(_: Option<i32>) -> AttributeType {
        AttributeType::I32
    }
}

unsafe impl Attribute for (i32, i32) {
    fn get_type(_: Option<(i32, i32)>) -> AttributeType {
        AttributeType::I32I32
    }
}

unsafe impl Attribute for [i32, ..2] {
    fn get_type(_: Option<[i32, ..2]>) -> AttributeType {
        AttributeType::I32I32
    }
}

unsafe impl Attribute for (i32, i32, i32) {
    fn get_type(_: Option<(i32, i32, i32)>) -> AttributeType {
        AttributeType::I32I32I32
    }
}

unsafe impl Attribute for [i32, ..3] {
    fn get_type(_: Option<[i32, ..3]>) -> AttributeType {
        AttributeType::I32I32I32
    }
}

unsafe impl Attribute for (i32, i32, i32, i32) {
    fn get_type(_: Option<(i32, i32, i32, i32)>) -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

unsafe impl Attribute for [i32, ..4] {
    fn get_type(_: Option<[i32, ..4]>) -> AttributeType {
        AttributeType::I32I32I32I32
    }
}

unsafe impl Attribute for u32 {
    fn get_type(_: Option<u32>) -> AttributeType {
        AttributeType::U32
    }
}

unsafe impl Attribute for (u32, u32) {
    fn get_type(_: Option<(u32, u32)>) -> AttributeType {
        AttributeType::U32U32
    }
}

unsafe impl Attribute for [u32, ..2] {
    fn get_type(_: Option<[u32, ..2]>) -> AttributeType {
        AttributeType::U32U32
    }
}

unsafe impl Attribute for (u32, u32, u32) {
    fn get_type(_: Option<(u32, u32, u32)>) -> AttributeType {
        AttributeType::U32U32U32
    }
}

unsafe impl Attribute for [u32, ..3] {
    fn get_type(_: Option<[u32, ..3]>) -> AttributeType {
        AttributeType::U32U32U32
    }
}

unsafe impl Attribute for (u32, u32, u32, u32) {
    fn get_type(_: Option<(u32, u32, u32, u32)>) -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

unsafe impl Attribute for [u32, ..4] {
    fn get_type(_: Option<[u32, ..4]>) -> AttributeType {
        AttributeType::U32U32U32U32
    }
}

unsafe impl Attribute for f32 {
    fn get_type(_: Option<f32>) -> AttributeType {
        AttributeType::F32
    }
}

unsafe impl Attribute for (f32, f32) {
    fn get_type(_: Option<(f32, f32)>) -> AttributeType {
        AttributeType::F32F32
    }
}

unsafe impl Attribute for [f32, ..2] {
    fn get_type(_: Option<[f32, ..2]>) -> AttributeType {
        AttributeType::F32F32
    }
}

unsafe impl Attribute for (f32, f32, f32) {
    fn get_type(_: Option<(f32, f32, f32)>) -> AttributeType {
        AttributeType::F32F32F32
    }
}

unsafe impl Attribute for [f32, ..3] {
    fn get_type(_: Option<[f32, ..3]>) -> AttributeType {
        AttributeType::F32F32F32
    }
}

unsafe impl Attribute for (f32, f32, f32, f32) {
    fn get_type(_: Option<(f32, f32, f32, f32)>) -> AttributeType {
        AttributeType::F32F32F32F32
    }
}

unsafe impl Attribute for [f32, ..4] {
    fn get_type(_: Option<[f32, ..4]>) -> AttributeType {
        AttributeType::F32F32F32F32
    }
}
