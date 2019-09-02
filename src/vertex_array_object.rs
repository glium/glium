use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem;

use smallvec::SmallVec;

use Handle;
use buffer::BufferAnySlice;
use program::Program;
use vertex::AttributeType;
use vertex::VertexFormat;
use GlObject;
use BufferExt;

use gl;
use context::CommandContext;
use version::Api;
use version::Version;

/// Stores and handles vertex attributes.
pub struct VertexAttributesSystem {
    // we maintain a list of VAOs for each vertexbuffer-indexbuffer-program association
    // the key is a (buffers-list-with-offset, program) ; the buffers list must be sorted
    vaos: RefCell<HashMap<(Vec<(gl::types::GLuint, usize)>, Handle), VertexArrayObject>>,
}

/// Object allowing one to bind vertex attributes to the current context.
pub struct Binder<'a, 'b, 'c: 'b> {
    context: &'b mut CommandContext<'c>,
    program: &'a Program,
    element_array_buffer: Option<BufferAnySlice<'a>>,
    vertex_buffers: SmallVec<[(gl::types::GLuint, VertexFormat, usize, usize, Option<u32>); 2]>,
    base_vertex: bool,
}

impl VertexAttributesSystem {
    /// Builds a new `VertexAttributesSystem`.
    #[inline]
    pub fn new() -> VertexAttributesSystem {
        VertexAttributesSystem {
            vaos: RefCell::new(HashMap::with_hasher(Default::default())),
        }
    }

    /// Starts the process of binding vertex attributes.
    ///
    /// `base_vertex` should be set to true if the backend supports the `glDraw*BaseVertex`
    /// functions. If `base_vertex` is true, then `bind` will return the base vertex to use.
    #[inline]
    pub fn start<'a, 'b, 'c: 'b>(ctxt: &'b mut CommandContext<'c>, program: &'a Program,
                                 indices: Option<BufferAnySlice<'a>>, base_vertex: bool)
                                 -> Binder<'a, 'b, 'c>
    {
        if let Some(indices) = indices {
            indices.prepare_for_element_array(ctxt);
        }

        Binder {
            context: ctxt,
            program: program,
            element_array_buffer: indices,
            vertex_buffers: SmallVec::new(),
            base_vertex: base_vertex,
        }
    }

    /// This function *must* be called whenever you destroy a buffer so that the system can
    /// purge its VAOs cache.
    #[inline]
    pub fn purge_buffer(ctxt: &mut CommandContext, id: gl::types::GLuint) {
        VertexAttributesSystem::purge_if(ctxt, |&(ref buffers, _)| {
            buffers.iter().find(|&&(b, _)| b == id).is_some()
        })
    }

    /// This function *must* be called whenever you destroy a program so that the system can
    /// purge its VAOs cache.
    #[inline]
    pub fn purge_program(ctxt: &mut CommandContext, program: Handle) {
        VertexAttributesSystem::purge_if(ctxt, |&(_, p)| p == program)
    }

    /// Purges the VAOs cache.
    pub fn purge_all(ctxt: &mut CommandContext) {
        let vaos = mem::replace(&mut *ctxt.vertex_array_objects.vaos.borrow_mut(),
                                HashMap::with_hasher(Default::default()));

        for (_, vao) in vaos {
            vao.destroy(ctxt);
        }
    }

    /// Purges the VAOs cache. Contrary to `purge_all`, this function expects the system to be
    /// destroyed soon.
    pub fn cleanup(ctxt: &mut CommandContext) {
        let vaos = mem::replace(&mut *ctxt.vertex_array_objects.vaos.borrow_mut(),
                                HashMap::with_hasher(Default::default()));

        for (_, vao) in vaos {
            vao.destroy(ctxt);
        }
    }

    /// Tells the VAOs system that the currently bound element array buffer will change.
    pub fn hijack_current_element_array_buffer(ctxt: &mut CommandContext) {
        let vaos = ctxt.vertex_array_objects.vaos.borrow_mut();

        for (_, vao) in vaos.iter() {
            if vao.id == ctxt.state.vertex_array {
                vao.element_array_buffer_hijacked.set(true);
                return;
            }
        }
    }

    /// Purges VAOs that match a certain condition.
    fn purge_if<F>(ctxt: &mut CommandContext, mut condition: F)
                   where F: FnMut(&(Vec<(gl::types::GLuint, usize)>, Handle)) -> bool
    {
        let mut vaos = ctxt.vertex_array_objects.vaos.borrow_mut();

        let mut keys = Vec::with_capacity(4);
        for (key, _) in &*vaos {
            if condition(key) {
                keys.push(key.clone());
            }
        }

        for key in keys {
            vaos.remove(&key).unwrap().destroy(ctxt);
        }
    }
}

impl<'a, 'b, 'c> Binder<'a, 'b, 'c> {
    /// Adds a buffer to bind as a source of vertices.
    ///
    /// # Parameters
    ///
    /// - `buffer`: The buffer to bind.
    /// - `first`: Offset of the first element of the buffer in number of elements.
    /// - `divisor`: If `Some`, use this value for `glVertexAttribDivisor` (instancing-related).
    #[inline]
    pub fn add(mut self, buffer: &BufferAnySlice, bindings: &VertexFormat, divisor: Option<u32>)
               -> Binder<'a, 'b, 'c>
    {
        let offset = buffer.get_offset_bytes();

        buffer.prepare_for_vertex_attrib_array(self.context);

        let (buffer, format, stride) = (buffer.get_id(), bindings.clone(),
                                        buffer.get_elements_size());

        self.vertex_buffers.push((buffer, format, offset, stride, divisor));
        self
    }

    /// Finish binding the vertex attributes.
    ///
    /// If `base_vertex` was set to true, returns the base vertex to use when drawing.
    pub fn bind(mut self) -> Option<gl::types::GLint> {
        let ctxt = self.context;

        if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.version >= &Version(Api::GlEs, 3, 0) ||
           ctxt.extensions.gl_arb_vertex_array_object || ctxt.extensions.gl_oes_vertex_array_object
           || ctxt.extensions.gl_apple_vertex_array_object
        {
            // VAOs are supported

            // finding the base vertex
            let base_vertex = if self.base_vertex {
                Some(self.vertex_buffers.iter()
                                        .filter(|&&(_, _, _, _, div)| div.is_none())
                                        .map(|&(_, _, off, stride, _)| off / stride)
                                        .min().unwrap_or(0))
            } else {
                None
            };

            // removing the offset corresponding to the base vertex
            if let Some(base_vertex) = base_vertex {
                for &mut (_, _, ref mut off, stride, _) in self.vertex_buffers.iter_mut() {
                    *off -= base_vertex * stride;
                }
            }

            let mut buffers_list: Vec<_> = self.vertex_buffers.iter()
                                                              .map(|&(v, _, o, s, _)| (v, o))
                                                              .collect();
            buffers_list.push((self.element_array_buffer.map(|b| b.get_id()).unwrap_or(0), 0));
            buffers_list.sort();

            let program_id = self.program.get_id();

            // trying to find an existing VAO in the cache
            if let Some(value) = ctxt.vertex_array_objects.vaos.borrow_mut()
                                     .get(&(buffers_list.clone(), program_id))
            {
                value.bind(ctxt);
                return base_vertex.map(|v| v as gl::types::GLint);
            }

            // if not found, building a new one
            let new_vao = unsafe {
                VertexArrayObject::new(ctxt, &self.vertex_buffers,
                                       self.element_array_buffer, self.program)
            };

            new_vao.bind(ctxt);
            ctxt.vertex_array_objects.vaos.borrow_mut().insert((buffers_list, program_id), new_vao);

            base_vertex.map(|v| v as gl::types::GLint)

        } else {
            // VAOs are not supported

            // just in case
            bind_vao(ctxt, 0);

            if let Some(element_array_buffer) = self.element_array_buffer {
                element_array_buffer.bind_to_element_array(ctxt);
            }

            for (vertex_buffer, bindings, offset, stride, divisor) in self.vertex_buffers.into_iter() {
                unsafe {
                    bind_attribute(ctxt, self.program, vertex_buffer, &bindings, offset, stride,
                                   divisor);
                }
            }

            // TODO: it is unlikely that a backend supports base vertex but not VAOs, so we just
            //       ignore this case ; however it would ideally be better to handle it
            if self.base_vertex {
                Some(0)
            } else {
                None
            }
        }
    }
}

/// Stores informations about how to bind a vertex buffer, an index buffer and a program.
struct VertexArrayObject {
    id: gl::types::GLuint,
    destroyed: bool,
    element_array_buffer: gl::types::GLuint,
    element_array_buffer_hijacked: Cell<bool>,
}

impl VertexArrayObject {
    /// Builds a new `VertexArrayObject`.
    ///
    /// The vertex buffer, index buffer and program must not outlive the
    /// VAO, and the VB & program attributes must not change.
    unsafe fn new(mut ctxt: &mut CommandContext,
                  vertex_buffers: &[(gl::types::GLuint, VertexFormat, usize, usize, Option<u32>)],
                  index_buffer: Option<BufferAnySlice>, program: &Program) -> VertexArrayObject
    {
        // checking the attributes types
        for &(_, ref bindings, _, _, _) in vertex_buffers {
            for &(ref name, _, ty, _) in bindings.iter() {
                let attribute = match program.get_attribute(Borrow::<str>::borrow(name)) {
                    Some(a) => a,
                    None => continue
                };

                if ty.get_num_components() != attribute.ty.get_num_components() ||
                    attribute.size != 1
                {
                    panic!("The program attribute `{}` does not match the vertex format. \
                            Program expected {:?}, got {:?}.", name, attribute.ty, ty);
                }
            }
        }

        // checking for missing attributes
        for (&ref name, _) in program.attributes() {
            let mut found = false;
            for &(_, ref bindings, _, _, _) in vertex_buffers {
                if bindings.iter().find(|&&(ref n, _, _, _)| n == name).is_some() {
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("The program attribute `{}` is missing in the vertex bindings", name);
            }
        };

        // TODO: check for collisions between the vertices sources

        // building the VAO
        let id = {
            let mut id = 0;
            if ctxt.version >= &Version(Api::Gl, 3, 0) ||
                ctxt.version >= &Version(Api::GlEs, 3, 0) ||
                ctxt.extensions.gl_arb_vertex_array_object
            {
                ctxt.gl.GenVertexArrays(1, &mut id);
            } else if ctxt.extensions.gl_oes_vertex_array_object {
                ctxt.gl.GenVertexArraysOES(1, &mut id);
            } else if ctxt.extensions.gl_apple_vertex_array_object {
                ctxt.gl.GenVertexArraysAPPLE(1, &mut id);
            } else {
                unreachable!();
            };
            id
        };

        // we don't use DSA as we're going to make multiple calls for this VAO
        // and we're likely going to use the VAO right after it's been created
        bind_vao(&mut ctxt, id);

        // binding index buffer
        if let Some(index_buffer) = index_buffer {
            index_buffer.bind_to_element_array(&mut ctxt);
        }

        for &(vertex_buffer, ref bindings, offset, stride, divisor) in vertex_buffers {
            bind_attribute(ctxt, program, vertex_buffer, bindings, offset, stride, divisor);
        }

        VertexArrayObject {
            id: id,
            destroyed: false,
            element_array_buffer: index_buffer.map(|b| b.get_id()).unwrap_or(0),
            element_array_buffer_hijacked: Cell::new(false),
        }
    }

    /// Sets this VAO as the current VAO.
    fn bind(&self, ctxt: &mut CommandContext) {
        unsafe {
            bind_vao(ctxt, self.id);

            if self.element_array_buffer_hijacked.get() {
                // TODO: use a proper function
                if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
                {
                    ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_array_buffer);
                } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                    ctxt.gl.BindBufferARB(gl::ELEMENT_ARRAY_BUFFER_ARB, self.element_array_buffer);
                } else {
                    unreachable!();
                }

                self.element_array_buffer_hijacked.set(false);
            }
        }
    }

    /// Must be called to destroy the VAO (otherwise its destructor will panic as a safety
    /// measure).
    fn destroy(mut self, ctxt: &mut CommandContext) {
        self.destroyed = true;

        // unbinding
        if ctxt.state.vertex_array == self.id {
            ctxt.state.vertex_array = 0;
        }

        // deleting
        if ctxt.version >= &Version(Api::Gl, 3, 0) ||
            ctxt.version >= &Version(Api::GlEs, 3, 0) ||
            ctxt.extensions.gl_arb_vertex_array_object
        {
            unsafe { ctxt.gl.DeleteVertexArrays(1, [ self.id ].as_ptr()) };
        } else if ctxt.extensions.gl_oes_vertex_array_object {
            unsafe { ctxt.gl.DeleteVertexArraysOES(1, [ self.id ].as_ptr()) };
        } else if ctxt.extensions.gl_apple_vertex_array_object {
            unsafe { ctxt.gl.DeleteVertexArraysAPPLE(1, [ self.id ].as_ptr()) };
        } else {
            unreachable!();
        }
    }
}

impl Drop for VertexArrayObject {
    #[inline]
    fn drop(&mut self) {
        assert!(self.destroyed);
    }
}

impl GlObject for VertexArrayObject {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

fn vertex_binding_type_to_gl(ty: AttributeType) -> (gl::types::GLenum, gl::types::GLint, gl::types::GLint) {
    match ty {
        AttributeType::I8 => (gl::BYTE, 1, 1),
        AttributeType::I8I8 => (gl::BYTE, 2, 1),
        AttributeType::I8I8I8 => (gl::BYTE, 3, 1),
        AttributeType::I8I8I8I8 => (gl::BYTE, 4, 1),
        AttributeType::U8 => (gl::UNSIGNED_BYTE, 1, 1),
        AttributeType::U8U8 => (gl::UNSIGNED_BYTE, 2, 1),
        AttributeType::U8U8U8 => (gl::UNSIGNED_BYTE, 3, 1),
        AttributeType::U8U8U8U8 => (gl::UNSIGNED_BYTE, 4, 1),
        AttributeType::I16 => (gl::SHORT, 1, 1),
        AttributeType::I16I16 => (gl::SHORT, 2, 1),
        AttributeType::I16I16I16 => (gl::SHORT, 3, 1),
        AttributeType::I16I16I16I16 => (gl::SHORT, 4, 1),
        AttributeType::U16 => (gl::UNSIGNED_SHORT, 1, 1),
        AttributeType::U16U16 => (gl::UNSIGNED_SHORT, 2, 1),
        AttributeType::U16U16U16 => (gl::UNSIGNED_SHORT, 3, 1),
        AttributeType::U16U16U16U16 => (gl::UNSIGNED_SHORT, 4, 1),
        AttributeType::I32 => (gl::INT, 1, 1),
        AttributeType::I32I32 => (gl::INT, 2, 1),
        AttributeType::I32I32I32 => (gl::INT, 3, 1),
        AttributeType::I32I32I32I32 => (gl::INT, 4, 1),
        AttributeType::U32 => (gl::UNSIGNED_INT, 1, 1),
        AttributeType::U32U32 => (gl::UNSIGNED_INT, 2, 1),
        AttributeType::U32U32U32 => (gl::UNSIGNED_INT, 3, 1),
        AttributeType::U32U32U32U32 => (gl::UNSIGNED_INT, 4, 1),
        AttributeType::I64 => (gl::INT64_NV, 1, 1),
        AttributeType::I64I64 => (gl::INT64_NV, 2, 1),
        AttributeType::I64I64I64 => (gl::INT64_NV, 3, 1),
        AttributeType::I64I64I64I64 => (gl::INT64_NV, 4, 1),
        AttributeType::U64 => (gl::UNSIGNED_INT64_NV, 1, 1),
        AttributeType::U64U64 => (gl::UNSIGNED_INT64_NV, 2, 1),
        AttributeType::U64U64U64 => (gl::UNSIGNED_INT64_NV, 3, 1),
        AttributeType::U64U64U64U64 => (gl::UNSIGNED_INT64_NV, 4, 1),
        AttributeType::F16 => (gl::HALF_FLOAT, 1, 1),
        AttributeType::F16F16 => (gl::HALF_FLOAT, 2, 1),
        AttributeType::F16F16F16 => (gl::HALF_FLOAT, 3, 1),
        AttributeType::F16F16F16F16 => (gl::HALF_FLOAT, 4, 1),
        AttributeType::F16x2x2 => (gl::HALF_FLOAT, 2, 2),
        AttributeType::F16x2x3 => (gl::HALF_FLOAT, 2, 3),
        AttributeType::F16x2x4 => (gl::HALF_FLOAT, 2, 4),
        AttributeType::F16x3x2 => (gl::HALF_FLOAT, 3, 2),
        AttributeType::F16x3x3 => (gl::HALF_FLOAT, 3, 3),
        AttributeType::F16x3x4 => (gl::HALF_FLOAT, 3, 4),
        AttributeType::F16x4x2 => (gl::HALF_FLOAT, 4, 2),
        AttributeType::F16x4x3 => (gl::HALF_FLOAT, 4, 3),
        AttributeType::F16x4x4 => (gl::HALF_FLOAT, 4, 4),
        AttributeType::F32 => (gl::FLOAT, 1, 1),
        AttributeType::F32F32 => (gl::FLOAT, 2, 1),
        AttributeType::F32F32F32 => (gl::FLOAT, 3, 1),
        AttributeType::F32F32F32F32 => (gl::FLOAT, 4, 1),
        AttributeType::F32x2x2 => (gl::FLOAT, 2, 2),
        AttributeType::F32x2x3 => (gl::FLOAT, 2, 3),
        AttributeType::F32x2x4 => (gl::FLOAT, 2, 4),
        AttributeType::F32x3x2 => (gl::FLOAT, 3, 2),
        AttributeType::F32x3x3 => (gl::FLOAT, 3, 3),
        AttributeType::F32x3x4 => (gl::FLOAT, 3, 4),
        AttributeType::F32x4x2 => (gl::FLOAT, 4, 2),
        AttributeType::F32x4x3 => (gl::FLOAT, 4, 3),
        AttributeType::F32x4x4 => (gl::FLOAT, 4, 4),
        AttributeType::F64 => (gl::DOUBLE, 1, 1),
        AttributeType::F64F64 => (gl::DOUBLE, 2, 1),
        AttributeType::F64F64F64 => (gl::DOUBLE, 3, 1),
        AttributeType::F64F64F64F64 => (gl::DOUBLE, 4, 1),
        AttributeType::F64x2x2 => (gl::DOUBLE, 2, 2),
        AttributeType::F64x2x3 => (gl::DOUBLE, 2, 3),
        AttributeType::F64x2x4 => (gl::DOUBLE, 2, 4),
        AttributeType::F64x3x2 => (gl::DOUBLE, 3, 2),
        AttributeType::F64x3x3 => (gl::DOUBLE, 3, 3),
        AttributeType::F64x3x4 => (gl::DOUBLE, 3, 4),
        AttributeType::F64x4x2 => (gl::DOUBLE, 4, 2),
        AttributeType::F64x4x3 => (gl::DOUBLE, 4, 3),
        AttributeType::F64x4x4 => (gl::DOUBLE, 4, 4),
        AttributeType::I2I10I10I10Reversed => (gl::INT_2_10_10_10_REV, 1, 1),
        AttributeType::U2U10U10U10Reversed => (gl::UNSIGNED_INT_2_10_10_10_REV, 1, 1),
        AttributeType::I10I10I10I2 => (gl::INT_10_10_10_2_OES, 1, 1),
        AttributeType::U10U10U10U2 => (gl::UNSIGNED_INT_10_10_10_2_OES, 1, 1),
        AttributeType::F10F11F11UnsignedIntReversed => (gl::UNSIGNED_INT_10F_11F_11F_REV, 1, 1),
        AttributeType::FixedFloatI16U16 => (gl::FIXED, 1, 1),
    }
}

/// Binds the vertex array object as the current one. Unbinds if `0` is passed.
///
/// ## Panic
///
/// Panics if the backend doesn't support vertex array objects.
fn bind_vao(ctxt: &mut CommandContext, vao_id: gl::types::GLuint) {
    if ctxt.state.vertex_array != vao_id {
        if ctxt.version >= &Version(Api::Gl, 3, 0) ||
            ctxt.version >= &Version(Api::GlEs, 3, 0) ||
            ctxt.extensions.gl_arb_vertex_array_object
        {
            unsafe { ctxt.gl.BindVertexArray(vao_id) };
        } else if ctxt.extensions.gl_oes_vertex_array_object {
            unsafe { ctxt.gl.BindVertexArrayOES(vao_id) };
        } else if ctxt.extensions.gl_apple_vertex_array_object {
            unsafe { ctxt.gl.BindVertexArrayAPPLE(vao_id) };
        } else {
            unreachable!();
        }

        ctxt.state.vertex_array = vao_id;
    }
}

/// Binds an individual attribute to the current VAO.
unsafe fn bind_attribute(ctxt: &mut CommandContext, program: &Program,
                         vertex_buffer: gl::types::GLuint, bindings: &VertexFormat,
                         buffer_offset: usize, stride: usize, divisor: Option<u32>)
{
    // glVertexAttribPointer uses the current array buffer
    // TODO: use a proper function
    if ctxt.state.array_buffer_binding != vertex_buffer {
        if ctxt.version >= &Version(Api::Gl, 1, 5) ||
            ctxt.version >= &Version(Api::GlEs, 2, 0)
        {
            ctxt.gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        } else if ctxt.extensions.gl_arb_vertex_buffer_object {
            ctxt.gl.BindBufferARB(gl::ARRAY_BUFFER_ARB, vertex_buffer);
        } else {
            unreachable!();
        }
        ctxt.state.array_buffer_binding = vertex_buffer;
    }

    // binding attributes
    for &(ref name, offset, ty, normalize) in bindings.iter() {
        let (data_type, elements_count, instances_count) = vertex_binding_type_to_gl(ty);

        let attribute = match program.get_attribute(Borrow::<str>::borrow(name)) {
            Some(a) => a,
            None => continue
        };

        if attribute.location != -1 {
            let (attribute_ty, _, _) = vertex_binding_type_to_gl(attribute.ty);
            if normalize {
                for i in 0..instances_count {
                    ctxt.gl.VertexAttribPointer((attribute.location + i) as u32,
                                                elements_count as gl::types::GLint, data_type, 1,
                                                stride as i32,
                                                (buffer_offset + offset + (i * elements_count * 4) as usize) as *const _)
                }
            } else {
                match attribute_ty {
                    gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT |
                    gl::INT | gl::UNSIGNED_INT =>
                        ctxt.gl.VertexAttribIPointer(attribute.location as u32,
                                                     elements_count as gl::types::GLint, data_type,
                                                     stride as i32,
                                                     (buffer_offset + offset) as *const _),

                    gl::FLOAT => {
                        for i in 0..instances_count {
                            ctxt.gl.VertexAttribPointer((attribute.location + i) as u32,
                                                        elements_count as gl::types::GLint, data_type, 0,
                                                        stride as i32,
                                                        (buffer_offset + offset + (i * elements_count * 4) as usize) as *const _)
                        }
                    },

                    gl::DOUBLE | gl::INT64_NV | gl::UNSIGNED_INT64_NV => {
                        for i in 0..instances_count {
                            ctxt.gl.VertexAttribLPointer((attribute.location + i) as u32,
                                                         elements_count as gl::types::GLint, data_type,
                                                         stride as i32,
                                                         (buffer_offset + offset + (i * elements_count * 8) as usize) as *const _)
                        }
                    },

                    _ => unreachable!()
                }
            }

            for i in 0..instances_count {
                if let Some(divisor) = divisor {
                    ctxt.gl.VertexAttribDivisor((attribute.location + i) as u32, divisor);
                }
                ctxt.gl.EnableVertexAttribArray((attribute.location + i) as u32);
            }
        }
    }
}
