use std::ptr;

use BufferExt;
use BufferSliceExt;
use ProgramExt;
use DrawError;
use UniformsExt;

use context::Context;
use ContextExt;
use TransformFeedbackSessionExt;

use fbo::{self, ValidatedAttachments};

use uniforms::Uniforms;
use {Program, ToGlEnum};
use index::{self, PrimitiveType, IndexType};
use vertex::{MultiVerticesSource, TransformFeedbackSession};
use vertex_array_object::VertexAttributesSystem;

use buffer::BufferAnySlice;
use vertex::VertexFormat;

use draw_parameters::DrawParameters;

use {gl, context, draw_parameters};
use version::Version;
use version::Api;

/// Describes the source to use for the vertices when drawing.
#[derive(Clone)]
pub struct VerticesSource<'a> {
    inner: VerticesSourceInner<'a>
}

impl<'a> VerticesSource<'a> {
    #[inline]
    pub unsafe fn from_buffer(buffer: BufferAnySlice<'a>, format: &'a VertexFormat,
                              per_instance: bool) -> VerticesSource<'a>
    {
        VerticesSource {
            inner: VerticesSourceInner::VertexBuffer(buffer, format, per_instance)
        }
    }

    /// Creates a marker for a number of vertices.
    #[inline]
    pub fn marker(len: usize, per_instance: bool) -> VerticesSource<'a> {
        VerticesSource {
            inner: VerticesSourceInner::Marker {
                len: len,
                per_instance: per_instance,
            }
        }
    }
}

#[derive(Clone)]
enum VerticesSourceInner<'a> {
    /// A buffer uploaded in the video memory.
    ///
    /// The second parameter is the number of vertices in the buffer.
    ///
    /// The third parameter tells whether or not this buffer is "per instance" (true) or
    /// "per vertex" (false).
    VertexBuffer(BufferAnySlice<'a>, &'a VertexFormat, bool),

    /// A marker indicating a "phantom list of attributes".
    Marker {
        /// Number of attributes.
        len: usize,

        /// Whether or not this buffer is "per instance" (true) or "per vertex" (false).
        per_instance: bool,
    },
}

/// Describes a source of indices used for drawing.
#[derive(Clone)]
pub struct IndicesSource<'a> {
    inner: IndicesSourceInner<'a>,
}

#[derive(Clone)]
pub enum IndicesSourceInner<'a> {
    /// A buffer uploaded in video memory.
    IndexBuffer {
        /// The buffer.
        buffer: BufferAnySlice<'a>,
        /// Type of indices in the buffer.
        data_type: IndexType,
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },

    /// Use a multidraw indirect buffer without indices.
    MultidrawArray {
        /// The buffer.
        buffer: BufferAnySlice<'a>,
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },

    /// Use a multidraw indirect buffer with indices.
    MultidrawElement {
        /// The buffer of the commands.
        commands: BufferAnySlice<'a>,
        /// The buffer of the indices.
        indices: BufferAnySlice<'a>,
        /// Type of indices in the buffer.
        data_type: IndexType,
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },

    /// Don't use indices. Assemble primitives by using the order in which the vertices are in
    /// the vertices source.
    NoIndices {
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },
}

impl<'a> IndicesSource<'a> {
    /// Builds a marker.
    #[inline]
    pub unsafe fn from_index_buffer(buffer: BufferAnySlice<'a>, data_type: IndexType,
                                    primitives: PrimitiveType) -> IndicesSource<'a>
    {
        IndicesSource {
            inner: IndicesSourceInner::IndexBuffer {
                buffer: buffer,
                data_type: data_type,
                primitives: primitives,
            }
        }
    }

    /// Builds a marker.
    #[inline]
    pub unsafe fn from_multidraw_array(buffer: BufferAnySlice<'a>, primitives: PrimitiveType)
                                       -> IndicesSource<'a>
    {
        IndicesSource {
            inner: IndicesSourceInner::MultidrawArray {
                buffer: buffer,
                primitives: primitives,
            }
        }
    }

    /// Builds a marker.
    #[inline]
    pub unsafe fn from_multidraw_element(commands: BufferAnySlice<'a>, indices: BufferAnySlice<'a>,
                                         data_type: IndexType, primitives: PrimitiveType)
                                         -> IndicesSource<'a>
    {
        IndicesSource {
            inner: IndicesSourceInner::MultidrawElement {
                commands: commands,
                indices: indices,
                data_type: data_type,
                primitives: primitives,
            }
        }
    }

    /// Builds a marker.
    #[inline]
    pub fn no_indices(primitives: PrimitiveType) -> IndicesSource<'a> {
        IndicesSource {
            inner: IndicesSourceInner::NoIndices { primitives: primitives }
        }
    }

    /// Returns the type of the primitives.
    #[inline]
    pub fn get_primitives_type(&self) -> PrimitiveType {
        match self.inner {
            IndicesSourceInner::IndexBuffer { primitives, .. } => primitives,
            IndicesSourceInner::MultidrawArray { primitives, .. } => primitives,
            IndicesSourceInner::MultidrawElement { primitives, .. } => primitives,
            IndicesSourceInner::NoIndices { primitives } => primitives,
        }
    }
}

/// Draws everything.
pub fn draw<'a, U, V>(context: &Context, framebuffer: Option<&ValidatedAttachments>,
                      vertex_buffers: V, indices: IndicesSource,
                      program: &Program, uniforms: &U, draw_parameters: &DrawParameters,
                      dimensions: (u32, u32)) -> Result<(), DrawError>
                      where U: Uniforms, V: MultiVerticesSource<'a>
{
    // this contains the list of fences that will need to be fulfilled after the draw command
    // has started
    let mut fences = Vec::with_capacity(0);

    // handling tessellation
    let vertices_per_patch = match indices.get_primitives_type() {
        index::PrimitiveType::Patches { vertices_per_patch } => {
            if let Some(max) = context.capabilities().max_patch_vertices {
                if vertices_per_patch == 0 || vertices_per_patch as gl::types::GLint > max {
                    return Err(DrawError::UnsupportedVerticesPerPatch);
                }
            } else {
                return Err(DrawError::TessellationNotSupported);
            }

            // TODO: programs created from binaries have the wrong value
            // for `has_tessellation_shaders`
            /*if !program.has_tessellation_shaders() {    // TODO:
                panic!("Default tessellation level is not supported yet");
            }*/

            Some(vertices_per_patch)
        },
        _ => {
            // TODO: programs created from binaries have the wrong value
            // for `has_tessellation_shaders`
            /*if program.has_tessellation_shaders() {
                return Err(DrawError::TessellationWithoutPatches);
            }*/

            None
        },
    };

    // starting the state changes
    let mut ctxt = context.make_current();

    // handling vertices source
    let (vertices_count, instances_count, base_vertex) = {
        let index_buffer = match indices.inner {
            IndicesSourceInner::IndexBuffer { buffer, .. } => Some(buffer),
            IndicesSourceInner::MultidrawArray { .. } => None,
            IndicesSourceInner::MultidrawElement { indices, .. } => Some(indices),
            IndicesSourceInner::NoIndices { .. } => None,
        };

        // determining whether we can use the `base_vertex` variants for drawing
        let use_base_vertex = match indices.inner {
            IndicesSourceInner::MultidrawArray { .. } => false,
            IndicesSourceInner::MultidrawElement { .. } => false,
            IndicesSourceInner::NoIndices { .. } => true,
            _ => ctxt.version >= &Version(Api::Gl, 3, 2) ||
                 ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                 ctxt.extensions.gl_arb_draw_elements_base_vertex ||
                 ctxt.extensions.gl_oes_draw_elements_base_vertex
        };

        // object that is used to build the bindings
        let mut binder = VertexAttributesSystem::start(&mut ctxt, program, index_buffer,
                                                       use_base_vertex);
        // number of vertices in the vertices sources, or `None` if there is a mismatch
        let mut vertices_count: Option<usize> = None;
        // number of instances to draw
        let mut instances_count: Option<usize> = None;

        for src in vertex_buffers.iter() {
            match src.inner {
                VerticesSourceInner::VertexBuffer(buffer, format, per_instance) => {
                    // TODO: assert!(buffer.get_elements_size() == total_size(format));

                    if let Some(fence) = buffer.add_fence() {
                        fences.push(fence);
                    }

                    binder = binder.add(&buffer, format, if per_instance { Some(1) } else { None });
                },
                _ => {}
            }

            match src.inner {
                VerticesSourceInner::VertexBuffer(ref buffer, _, false) => {
                    if let Some(curr) = vertices_count {
                        if curr != buffer.get_elements_count() {
                            vertices_count = None;
                            break;
                        }
                    } else {
                        vertices_count = Some(buffer.get_elements_count());
                    }
                },
                VerticesSourceInner::VertexBuffer(ref buffer, _, true) => {
                    if let Some(curr) = instances_count {
                        if curr != buffer.get_elements_count() {
                            return Err(DrawError::InstancesCountMismatch);
                        }
                    } else {
                        instances_count = Some(buffer.get_elements_count());
                    }
                },
                VerticesSourceInner::Marker { len, per_instance } if !per_instance => {
                    if let Some(curr) = vertices_count {
                        if curr != len {
                            vertices_count = None;
                            break;
                        }
                    } else {
                        vertices_count = Some(len);
                    }
                },
                VerticesSourceInner::Marker { len, per_instance } if per_instance => {
                    if let Some(curr) = instances_count {
                        if curr != len {
                            return Err(DrawError::InstancesCountMismatch);
                        }
                    } else {
                        instances_count = Some(len);
                    }
                },
                _ => ()
            }
        }

        (vertices_count, instances_count, binder.bind().unwrap_or(0))
    };

    // binding the FBO to draw upon
    {
        let fbo_id = fbo::FramebuffersContainer::get_framebuffer_for_drawing(&mut ctxt, framebuffer);
        unsafe { fbo::bind_framebuffer(&mut ctxt, fbo_id, true, false) };
    };

    // binding the program and uniforms
    program.use_program(&mut ctxt);
    try!(uniforms.bind_uniforms(&mut ctxt, program, &mut fences));

    // sync-ing draw_parameters
    unsafe {
        try!(draw_parameters::sync(&mut ctxt, draw_parameters, dimensions, indices.get_primitives_type()));
        sync_vertices_per_patch(&mut ctxt, vertices_per_patch);

        // TODO: make sure that the program is the right one
        // TODO: changing the current transform feedback requires pausing/unbinding before changing the program
        if let Some(ref tf) = draw_parameters.transform_feedback {
            tf.bind(&mut ctxt, indices.get_primitives_type());
        } else {
            TransformFeedbackSession::unbind(&mut ctxt);
        }
    }

    // drawing
    // TODO: make this code more readable
    {
        match indices.inner {
            IndicesSourceInner::IndexBuffer { ref buffer, data_type, primitives } => {
                let ptr: *const u8 = ptr::null_mut();
                let ptr = unsafe { ptr.offset(buffer.get_offset_bytes() as isize) };

                if let Some(fence) = buffer.add_fence() {
                    fences.push(fence);
                }

                unsafe {
                    if let Some(instances_count) = instances_count {
                        if base_vertex != 0 {
                            if ctxt.version >= &Version(Api::Gl, 3, 2) ||
                               ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                               ctxt.extensions.gl_arb_draw_elements_base_vertex
                            {
                                ctxt.gl.DrawElementsInstancedBaseVertex(primitives.to_glenum(),
                                                                     buffer.get_elements_count() as
                                                                        gl::types::GLsizei,
                                                                        data_type.to_glenum(),
                                                                        ptr as *const _,
                                                                        instances_count as
                                                                        gl::types::GLsizei,
                                                                        base_vertex);

                            } else if ctxt.extensions.gl_oes_draw_elements_base_vertex {
                                ctxt.gl.DrawElementsInstancedBaseVertexOES(primitives.to_glenum(),
                                                                     buffer.get_elements_count() as
                                                                           gl::types::GLsizei,
                                                                           data_type.to_glenum(),
                                                                        ptr as *const _,
                                                                           instances_count as
                                                                           gl::types::GLsizei,
                                                                           base_vertex);
                            } else {
                                unreachable!();
                            }

                        } else {
                            ctxt.gl.DrawElementsInstanced(primitives.to_glenum(),
                                                          buffer.get_elements_count() as
                                                          gl::types::GLsizei,
                                                          data_type.to_glenum(),
                                                          ptr as *const _,
                                                          instances_count as gl::types::GLsizei);
                        }

                    } else {
                        if base_vertex != 0 {
                            if ctxt.version >= &Version(Api::Gl, 3, 2) ||
                               ctxt.version >= &Version(Api::GlEs, 3, 2) ||
                               ctxt.extensions.gl_arb_draw_elements_base_vertex
                            {
                                ctxt.gl.DrawElementsBaseVertex(primitives.to_glenum(),
                                                               buffer.get_elements_count() as
                                                               gl::types::GLsizei,
                                                               data_type.to_glenum(),
                                                               ptr as *const _,
                                                               base_vertex);

                            } else if ctxt.extensions.gl_oes_draw_elements_base_vertex {
                                ctxt.gl.DrawElementsBaseVertexOES(primitives.to_glenum(),
                                                                  buffer.get_elements_count() as
                                                                  gl::types::GLsizei,
                                                                  data_type.to_glenum(),
                                                                  ptr as *const _,
                                                                  base_vertex);
                            } else {
                                unreachable!();
                            }

                        } else {
                            ctxt.gl.DrawElements(primitives.to_glenum(),
                                                 buffer.get_elements_count() as gl::types::GLsizei,
                                                 data_type.to_glenum(),
                                                 ptr as *const _);
                        }
                    }
                }
            },

            IndicesSourceInner::MultidrawArray { ref buffer, primitives } => {
                let ptr: *const u8 = ptr::null_mut();
                let ptr = unsafe { ptr.offset(buffer.get_offset_bytes() as isize) };

                debug_assert_eq!(base_vertex, 0);       // enforced earlier in this function

                if let Some(fence) = buffer.add_fence() {
                    fences.push(fence);
                }

                unsafe {
                    buffer.prepare_and_bind_for_draw_indirect(&mut ctxt);
                    ctxt.gl.MultiDrawArraysIndirect(primitives.to_glenum(), ptr as *const _,
                                                    buffer.get_elements_count() as gl::types::GLsizei,
                                                    0);
                }
            },

            IndicesSourceInner::MultidrawElement { ref commands, ref indices, data_type, primitives } => {
                let cmd_ptr: *const u8 = ptr::null_mut();
                let cmd_ptr = unsafe { cmd_ptr.offset(commands.get_offset_bytes() as isize) };

                if let Some(fence) = commands.add_fence() {
                    fences.push(fence);
                }

                if let Some(fence) = indices.add_fence() {
                    fences.push(fence);
                }

                unsafe {
                    commands.prepare_and_bind_for_draw_indirect(&mut ctxt);
                    debug_assert_eq!(base_vertex, 0);       // enforced earlier in this function
                    ctxt.gl.MultiDrawElementsIndirect(primitives.to_glenum(), data_type.to_glenum(),
                                                      cmd_ptr as *const _,
                                                      commands.get_elements_count() as gl::types::GLsizei,
                                                      0);
                }
            },

            IndicesSourceInner::NoIndices { primitives } => {
                let vertices_count = match vertices_count {
                    Some(c) => c,
                    None => return Err(DrawError::VerticesSourcesLengthMismatch)
                };

                unsafe {
                    if let Some(instances_count) = instances_count {
                        ctxt.gl.DrawArraysInstanced(primitives.to_glenum(), base_vertex,
                                                    vertices_count as gl::types::GLsizei,
                                                    instances_count as gl::types::GLsizei);
                    } else {
                        ctxt.gl.DrawArrays(primitives.to_glenum(), base_vertex,
                                           vertices_count as gl::types::GLsizei);
                    }
                }
            },
        };
    };

    ctxt.state.next_draw_call_id += 1;

    // fulfilling the fences
    for fence in fences.into_iter() {
        fence.insert(&mut ctxt);
    }

    Ok(())
}

unsafe fn sync_vertices_per_patch(ctxt: &mut context::CommandContext, vertices_per_patch: Option<u16>) {
    if let Some(vertices_per_patch) = vertices_per_patch {
        let vertices_per_patch = vertices_per_patch as gl::types::GLint;
        if ctxt.state.patch_patch_vertices != vertices_per_patch {
            ctxt.gl.PatchParameteri(gl::PATCH_VERTICES, vertices_per_patch);
            ctxt.state.patch_patch_vertices = vertices_per_patch;
        }
    }
}
