/*!
In order to draw, you need to provide a source of indices which is used to link the vertices
together into *primitives*.

There are height types of primitives, each one with a corresponding struct:
 - `PointsList`
 - `LinesList`
 - `LinesListAdjacency`
 - `LineStrip`
 - `LineStripAdjacency`
 - `TrianglesList`
 - `TrianglesListAdjacency`
 - `TriangleStrip`
 - `TriangleStripAdjacency`
 - `TriangleFan`

Each struct contains a vector with the indices and can be used as an `IndicesSource`.

However the most optimal way to draw something is to load the data in the video memory by
creating an `IndexBuffer`.

*/
use buffer::{mod, Buffer};
use gl;
use GlObject;
use ToGlEnum;

/// Can be used as a source of indices when drawing.
pub trait ToIndicesSource<D> {
    /// Builds the `IndicesSource`.
    fn to_indices_source<'a>(&'a self) -> IndicesSource<'a, D>;
}

/// Describes a source of indices used for drawing.
#[deriving(Clone)]
pub enum IndicesSource<'a, T: 'a> {
    /// A buffer uploaded in video memory.
    IndexBuffer {
        /// The buffer.
        buffer: &'a IndexBuffer,
        /// Offset of the first element of the buffer to use.
        offset: uint,
        /// Number of elements in the buffer to use.
        length: uint,
    },

    /// A buffer in RAM.
    Buffer {
        /// Slice of data to use.
        pointer: &'a [T],
        /// Type of primitives contained in the buffer.
        primitives: PrimitiveType,
        /// Offset of the first element of the buffer to use.
        offset: uint,
        /// Number of elements in the buffer to use.
        length: uint,
    }
}

impl<'a, T> IndicesSource<'a, T> where T: Index {
    /// Returns the types of primitives.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        match self {
            &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_primitives_type(),
            &IndicesSource::Buffer { primitives, .. } => primitives,
        }
    }

    /// Returns the types of indices.
    pub fn get_indices_type(&self) -> IndexType {
        match self {
            &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_indices_type(),
            &IndicesSource::Buffer { .. } => Index::get_type(None::<T>),
        }
    }

    /// Returns the first element to use from the buffer.
    pub fn get_offset(&self) -> uint {
        match self {
            &IndicesSource::IndexBuffer { offset, .. } => offset,
            &IndicesSource::Buffer { offset, .. } => offset,
        }
    }

    /// Returns the lgnth of the buffer to use.
    pub fn get_length(&self) -> uint {
        match self {
            &IndicesSource::IndexBuffer { length, .. } => length,
            &IndicesSource::Buffer { length, .. } => length,
        }
    }
}

/// List of available primitives.
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    /// 
    Points,
    /// 
    LinesList,
    /// 
    LinesListAdjacency,
    /// 
    LineStrip,
    /// 
    LineStripAdjacency,
    /// 
    TrianglesList,
    /// 
    TrianglesListAdjacency,
    /// 
    TriangleStrip,
    /// 
    TriangleStripAdjacency,
    /// 
    TriangleFan,
}

impl ToGlEnum for PrimitiveType {
    fn to_glenum(&self) -> gl::types::GLenum {
        match self {
            &PrimitiveType::Points => gl::POINTS,
            &PrimitiveType::LinesList => gl::LINES,
            &PrimitiveType::LinesListAdjacency => gl::LINES_ADJACENCY,
            &PrimitiveType::LineStrip => gl::LINE_STRIP,
            &PrimitiveType::LineStripAdjacency => gl::LINE_STRIP_ADJACENCY,
            &PrimitiveType::TrianglesList => gl::TRIANGLES,
            &PrimitiveType::TrianglesListAdjacency => gl::TRIANGLES_ADJACENCY,
            &PrimitiveType::TriangleStrip => gl::TRIANGLE_STRIP,
            &PrimitiveType::TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
            &PrimitiveType::TriangleFan => gl::TRIANGLE_FAN,
        }
    }
}

/// A list of indices loaded in the graphics card's memory.
#[deriving(Show)]
pub struct IndexBuffer {
    buffer: Buffer,
    data_type: IndexType,
    primitives: PrimitiveType,
}

impl IndexBuffer {
    /// Builds a new index buffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate glium;
    /// # extern crate glutin;
    /// # fn main() {
    /// # let display: glium::Display = unsafe { ::std::mem::uninitialized() };
    /// let index_buffer = glium::IndexBuffer::new(&display,
    ///     glium::index_buffer::TrianglesList(vec![0u8, 1, 2, 1, 3, 4, 2, 4, 3]));
    /// # }
    /// ```
    ///
    /// # Panic
    ///
    /// Attempting to draw with an index buffer that uses an indices format with adjacency infos
    /// on OpenGL ES will trigger a panic.
    ///
    /// If you want to be compatible with all platforms, it is preferable to disable the
    /// `gl_extensions` feature, which prevents you from accidentally using them.
    ///
    pub fn new<T: IntoIndexBuffer>(display: &super::Display, data: T) -> IndexBuffer {
        data.into_index_buffer(display)
    }

    /// Returns the type of primitives associated with this index buffer.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        self.primitives
    }

    /// Returns the data type of the indices inside this index buffer.
    pub fn get_indices_type(&self) -> IndexType {
        self.data_type
    }
}

impl GlObject for IndexBuffer {
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_id()
    }
}

impl ToIndicesSource<u16> for IndexBuffer {      // TODO: u16?
    fn to_indices_source(&self) -> IndicesSource<u16> {     // TODO: ?
        IndicesSource::IndexBuffer {
            buffer: self,
            offset: 0,
            length: self.buffer.get_elements_count() as uint,
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        // removing VAOs which contain this index buffer
        let mut vaos = self.buffer.get_display().vertex_array_objects.lock().unwrap();
        let to_delete = vaos.keys().filter(|&&(_, i, _)| i == self.buffer.get_id())
            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
    }
}

/// Types of indices in an indices source.
#[deriving(Show, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// u8
    U8,
    /// u16
    U16,
    /// u32
    U32,
}

impl ToGlEnum for IndexType {
    fn to_glenum(&self) -> gl::types::GLenum {
        match self {
            &IndexType::U8 => gl::UNSIGNED_BYTE,
            &IndexType::U16 => gl::UNSIGNED_SHORT,
            &IndexType::U32 => gl::UNSIGNED_INT,
        }
    }
}

/// An index from the index buffer.
pub unsafe trait Index: Copy + Send {
    /// Returns the `IndexType` corresponding to this type.
    fn get_type(Option<Self>) -> IndexType;
}

unsafe impl Index for u8 {
    fn get_type(_: Option<u8>) -> IndexType {
        IndexType::U8
    }
}

unsafe impl Index for u16 {
    fn get_type(_: Option<u16>) -> IndexType {
        IndexType::U16
    }
}

unsafe impl Index for u32 {
    fn get_type(_: Option<u32>) -> IndexType {
        IndexType::U32
    }
}

/// Object is convertible to an index buffer.
pub trait IntoIndexBuffer {
    /// Creates a new `IndexBuffer` with the list of indices.
    fn into_index_buffer(self, &super::Display) -> IndexBuffer;
}

/// A list of points stored in RAM.
#[deriving(Clone)]
pub struct PointsList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for PointsList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");

        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::Points,
        }
    }
}

impl<T> ToIndicesSource<T> for PointsList<T> where T: Index + Send + Copy {
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::Points,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines stored in RAM.
pub struct LinesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::LinesList,
        }
    }
}

impl<T> ToIndicesSource<T> for LinesList<T> where T: Index + Send + Copy {
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LinesList,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct LinesListAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::LinesListAdjacency,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> ToIndicesSource<T> for LinesListAdjacency<T> where T: Index + Send + Copy {    
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LinesListAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines connected together stored in RAM.
pub struct LineStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::LineStrip,
        }
    }
}

impl<T> ToIndicesSource<T> for LineStrip<T> where T: Index + Send + Copy {    
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LineStrip,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines connected together with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct LineStripAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::LineStripAdjacency,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> ToIndicesSource<T> for LineStripAdjacency<T> where T: Index + Send + Copy {    
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LineStripAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TrianglesList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::TrianglesList,
        }
    }
}

impl<T> ToIndicesSource<T> for TrianglesList<T> where T: Index + Send + Copy {    
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TrianglesList,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct TrianglesListAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::TrianglesListAdjacency,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> ToIndicesSource<T> for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TrianglesListAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles connected together stored in RAM.
pub struct TriangleStrip<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStrip<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::TriangleStrip,
        }
    }
}

impl<T> ToIndicesSource<T> for TriangleStrip<T> where T: Index + Send + Copy {
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleStrip,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles connected together with adjacency infos stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency infos. Attempting to use this type while
/// drawing will thus panic.
/// If you want to be compatible with all platforms, it is preferable to disable the
/// `gl_extensions` feature.
///
/// # Features
///
/// Only available if the `gl_extensions` feature is enabled.
#[cfg(feature = "gl_extensions")]
pub struct TriangleStripAdjacency<T>(pub Vec<T>);

#[cfg(feature = "gl_extensions")]
impl<T> IntoIndexBuffer for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::TriangleStripAdjacency,
        }
    }
}

#[cfg(feature = "gl_extensions")]
impl<T> ToIndicesSource<T> for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleStripAdjacency,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles stored in RAM.
pub struct TriangleFan<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleFan<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, gl::STATIC_DRAW),
            data_type: Index::get_type(None::<T>),
            primitives: PrimitiveType::TriangleFan,
        }
    }
}

impl<T> ToIndicesSource<T> for TriangleFan<T> where T: Index + Send + Copy {
    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleFan,
            offset: 0,
            length: self.0.len(),
        }
    }
}
