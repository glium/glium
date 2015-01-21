/*!
In order to draw, you need to provide a source of indices which is used to link the vertices
together into *primitives*.

There are ten types of primitives, each one with a corresponding struct:
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
use buffer::{self, Buffer};
use gl;
use GlObject;
use ToGlEnum;

use sync::LinearSyncFence;
use std::sync::mpsc::Sender;

/// Can be used as a source of indices when drawing.
pub trait ToIndicesSource {
    /// The type of data.
    type Data: Index;

    /// Builds the `IndicesSource`.
    fn to_indices_source<'a>(&'a self) -> IndicesSource<'a, Self::Data>;
}

/// Describes a source of indices used for drawing.
#[derive(Clone)]
pub enum IndicesSource<'a, T: 'a> {
    /// A buffer uploaded in video memory.
    IndexBuffer {
        /// The buffer.
        buffer: &'a IndexBuffer,
        /// Sender which must be used to send back a fence that is signaled when the buffer has
        /// finished being used.
        fence: Option<Sender<LinearSyncFence>>,
        /// Offset of the first element of the buffer to use.
        offset: usize,
        /// Number of elements in the buffer to use.
        length: usize,
    },

    /// A buffer in RAM.
    Buffer {
        /// Slice of data to use.
        pointer: &'a [T],
        /// Type of primitives contained in the buffer.
        primitives: PrimitiveType,
        /// Offset of the first element of the buffer to use.
        offset: usize,
        /// Number of elements in the buffer to use.
        length: usize,
    }
}

impl<'a, T> IndicesSource<'a, T> where T: Index {
    /// Returns the type of the primitives.
    pub fn get_primitives_type(&self) -> PrimitiveType {
        match self {
            &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_primitives_type(),
            &IndicesSource::Buffer { primitives, .. } => primitives,
        }
    }

    /// Returns the type of the indices.
    pub fn get_indices_type(&self) -> IndexType {
        match self {
            &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_indices_type(),
            &IndicesSource::Buffer { .. } => <T as Index>::get_type(),
        }
    }

    /// Returns the first element to use from the buffer.
    pub fn get_offset(&self) -> usize {
        match self {
            &IndicesSource::IndexBuffer { offset, .. } => offset,
            &IndicesSource::Buffer { offset, .. } => offset,
        }
    }

    /// Returns the length of the buffer.
    pub fn get_length(&self) -> usize {
        match self {
            &IndicesSource::IndexBuffer { length, .. } => length,
            &IndicesSource::Buffer { length, .. } => length,
        }
    }
}

/// List of available primitives.
#[derive(Show, Clone, Copy, PartialEq, Eq)]
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
    ///
    Patches {
        /// Number of vertices per patch.
        vertices_per_patch: u16,
    },
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
            &PrimitiveType::Patches { .. } => gl::PATCHES,
        }
    }
}

/// A list of indices loaded in the graphics card's memory.
#[derive(Show)]
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
    /// On OpenGL ES, attempting to draw with an index buffer that uses an index
    /// format with adjacency information will trigger a panic.
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

impl ToIndicesSource for IndexBuffer {
    type Data = u16;      // TODO: u16?

    fn to_indices_source(&self) -> IndicesSource<u16> {     // TODO: u16?
        let fence = if self.buffer.is_persistent() {
            Some(self.buffer.add_fence())
        } else {
            None
        };

        IndicesSource::IndexBuffer {
            buffer: self,
            fence: fence,
            offset: 0,
            length: self.buffer.get_elements_count() as usize,
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        // removing VAOs which contain this index buffer
        let mut vaos = self.buffer.get_display().context.vertex_array_objects.lock().unwrap();
        let to_delete = vaos.keys()
                            .filter(|&&(ref bufs, _)| {
                                bufs.iter().find(|&&b| b == self.buffer.get_id()).is_some()
                            })
                            .map(|k| k.clone()).collect::<Vec<_>>();
        for k in to_delete.into_iter() {
            vaos.remove(&k);
        }
    }
}

/// Type of the indices in an index source.
#[derive(Show, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]    // GLenum
pub enum IndexType {
    /// u8
    U8 = gl::UNSIGNED_BYTE,
    /// u16
    U16 = gl::UNSIGNED_SHORT,
    /// u32
    U32 = gl::UNSIGNED_INT,
}

impl ToGlEnum for IndexType {
    fn to_glenum(&self) -> gl::types::GLenum {
        *self as gl::types::GLenum
    }
}

/// An index from the index buffer.
pub unsafe trait Index: Copy + Send {
    /// Returns the `IndexType` corresponding to this type.
    fn get_type() -> IndexType;
}

unsafe impl Index for u8 {
    fn get_type() -> IndexType {
        IndexType::U8
    }
}

unsafe impl Index for u16 {
    fn get_type() -> IndexType {
        IndexType::U16
    }
}

unsafe impl Index for u32 {
    fn get_type() -> IndexType {
        IndexType::U32
    }
}

/// Object that is convertible to an index buffer.
pub trait IntoIndexBuffer {
    /// Creates a new `IndexBuffer` with the list of indices.
    fn into_index_buffer(self, &super::Display) -> IndexBuffer;
}

/// A list of points stored in RAM.
#[derive(Clone)]
pub struct PointsList<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for PointsList<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");

        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::Points,
        }
    }
}

impl<T> ToIndicesSource for PointsList<T> where T: Index + Send + Copy {
    type Data = T;

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
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::LinesList,
        }
    }
}

impl<T> ToIndicesSource for LinesList<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LinesList,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct LinesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LinesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::LinesListAdjacency,
        }
    }
}

impl<T> ToIndicesSource for LinesListAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

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
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::LineStrip,
        }
    }
}

impl<T> ToIndicesSource for LineStrip<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::LineStrip,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of lines connected together, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct LineStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for LineStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::LineStripAdjacency,
        }
    }
}

impl<T> ToIndicesSource for LineStripAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

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
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::TrianglesList,
        }
    }
}

impl<T> ToIndicesSource for TrianglesList<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TrianglesList,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct TrianglesListAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::TrianglesListAdjacency,
        }
    }
}

impl<T> ToIndicesSource for TrianglesListAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

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
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::TriangleStrip,
        }
    }
}

impl<T> ToIndicesSource for TriangleStrip<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleStrip,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of triangles connected together, with adjacency information, stored in RAM.
///
/// # Panic
///
/// OpenGL ES doesn't support adjacency information. Attempting to use this type while
/// drawing will thus panic.
pub struct TriangleStripAdjacency<T>(pub Vec<T>);

impl<T> IntoIndexBuffer for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::TriangleStripAdjacency,
        }
    }
}

impl<T> ToIndicesSource for TriangleStripAdjacency<T> where T: Index + Send + Copy {
    type Data = T;

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
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::TriangleFan,
        }
    }
}

impl<T> ToIndicesSource for TriangleFan<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::TriangleFan,
            offset: 0,
            length: self.0.len(),
        }
    }
}

/// A list of patches stored in RAM.
///
/// The second parameter is the number of vertices per patch.
pub struct Patches<T>(pub Vec<T>, pub u16);

impl<T> IntoIndexBuffer for Patches<T> where T: Index + Send + Copy {
    fn into_index_buffer(self, display: &super::Display) -> IndexBuffer {
        use std::mem;
        assert!(mem::align_of::<T>() <= mem::size_of::<T>(), "Buffer elements are not \
                                                              packed in memory");
        IndexBuffer {
            buffer: Buffer::new::<buffer::ArrayBuffer, _>(display, self.0, false),
            data_type: <T as Index>::get_type(),
            primitives: PrimitiveType::Patches { vertices_per_patch: self.1 },
        }
    }
}

impl<T> ToIndicesSource for Patches<T> where T: Index + Send + Copy {
    type Data = T;

    fn to_indices_source(&self) -> IndicesSource<T> {
        IndicesSource::Buffer {
            pointer: self.0.as_slice(),
            primitives: PrimitiveType::Patches { vertices_per_patch: self.1 },
            offset: 0,
            length: self.0.len(),
        }
    }
}
