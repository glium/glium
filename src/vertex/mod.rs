/*!
Contains everything related to vertex buffers.

The main struct is the `VertexBuffer`, which represents a buffer in the video memory,
containing a list of vertices.

In order to create a vertex buffer, you must first create a struct that represents each vertex,
and implement the `glium::vertex::Vertex` trait on it. The `#[vertex_format]` attribute
located in `glium_macros` helps you with that.

```
# #![feature(plugin)]
# #[plugin]
# extern crate glium_macros;
# extern crate glium;
# extern crate glutin;
# fn main() {
#[vertex_format]
#[derive(Copy)]
struct Vertex {
    position: [f32; 3],
    texcoords: [f32; 2],
}
# }
```

Next, build a `Vec` of the vertices that you want to upload, and pass it to
`VertexBuffer::new`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# #[derive(Copy)]
# struct Vertex {
#     position: [f32; 3],
#     texcoords: [f32; 2],
# }
# impl glium::vertex::Vertex for Vertex {
#     fn build_bindings(_: Option<Vertex>) -> glium::vertex::VertexFormat {
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

let vertex_buffer = glium::vertex::VertexBuffer::new(&display, data);
```

*/
use std::sync::mpsc::Sender;
use sync::LinearSyncFence;

pub use self::buffer::{VertexBuffer, VertexBufferAny, Mapping};
pub use self::format::{AttributeType, VertexFormat};

mod buffer;
mod format;

/// Describes the source to use for the vertices when drawing.
#[derive(Clone)]
pub enum VerticesSource<'a> {
    /// A buffer uploaded in the video memory.
    ///
    /// If the second parameter is `Some`, then a fence *must* be sent with this sender for
    /// when the buffer stops being used.
    VertexBuffer(&'a VertexBufferAny, Option<Sender<LinearSyncFence>>),
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
