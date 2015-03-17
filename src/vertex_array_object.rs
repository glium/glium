use std::borrow::Borrow;
use std::cell::RefCell;
use std::default::Default;
use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::mem;

use Handle;
use program::Program;
use index::IndicesSource;
use vertex::{VerticesSource, AttributeType};
use GlObject;

use {libc, gl, context};
use context::CommandContext;
use version::Api;
use util;

/// Stores and handles vertex attributes.
pub struct VertexAttributesSystem {
    // we maintain a list of VAOs for each vertexbuffer-indexbuffer-program association
    // the key is a (buffers-list, program) ; the buffers list must be sorted
    vaos: RefCell<HashMap<(Vec<gl::types::GLuint>, Handle),
                          VertexArrayObject, DefaultState<util::FnvHasher>>>,
}

impl VertexAttributesSystem {
    pub fn new() -> VertexAttributesSystem {
        VertexAttributesSystem {
            vaos: RefCell::new(HashMap::with_hash_state(Default::default())),
        }
    }

    /// Makes sure that the VAO currently binded contains the right information.
    pub fn bind_vao<I>(&self, ctxt: &mut CommandContext, vertex_buffers: &[&VerticesSource],
                       indices: &IndicesSource<I>, program: &Program)
                       where I: ::index::Index
    {
        let ib_id = match indices {
            &IndicesSource::Buffer { .. } => 0,
            &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_id(),
            &IndicesSource::NoIndices { .. } => 0,
        };

        let buffers_list = {
            let mut buffers_list = Vec::with_capacity(1 + vertex_buffers.len());
            if ib_id != 0 {
                buffers_list.push(ib_id);
            }
            for vertex_buffer in vertex_buffers.iter() {
                buffers_list.push(match vertex_buffer {
                    &&VerticesSource::VertexBuffer(ref vb, _, _) => vb.get_id(),
                    &&VerticesSource::PerInstanceBuffer(ref buf) => buf.get_id(),
                });
            }
            buffers_list.sort();
            buffers_list
        };

        let program_id = program.get_id();

        if let Some(value) = self.vaos.borrow_mut()
                                 .get(&(buffers_list.clone(), program_id))
        {
            bind_vao(ctxt, value.id);
            return;
        }

        // we create the new VAO without the mutex locked
        let new_vao = VertexArrayObject::new(ctxt, vertex_buffers, ib_id, program);
        bind_vao(ctxt, new_vao.id);
        self.vaos.borrow_mut().insert((buffers_list, program_id), new_vao);
    }

    pub fn purge_buffer(&self, ctxt: &mut CommandContext, id: gl::types::GLuint) {
        self.purge_if(ctxt, |&(ref buffers, _)| {
            buffers.iter().find(|&b| b == &id).is_some()
        })
    }

    pub fn purge_program(&self, ctxt: &mut CommandContext, program: Handle) {
        self.purge_if(ctxt, |&(_, p)| p == program)
    }

    pub fn purge_all(&self, ctxt: &mut CommandContext) {
        let vaos = mem::replace(&mut *self.vaos.borrow_mut(),
                                HashMap::with_hash_state(Default::default()));

        for (_, vao) in vaos {
            vao.destroy(ctxt);
        }
    }

    pub fn cleanup(&mut self, ctxt: &mut CommandContext) {
        let vaos = mem::replace(&mut *self.vaos.borrow_mut(),
                                HashMap::with_capacity_and_hash_state(0, Default::default()));

        for (_, vao) in vaos {
            vao.destroy(ctxt);
        }
    }

    fn purge_if<F>(&self, ctxt: &mut CommandContext, mut condition: F)
                   where F: FnMut(&(Vec<gl::types::GLuint>, Handle)) -> bool
    {
        let mut vaos = self.vaos.borrow_mut();

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

/// Stores informations about how to bind a vertex buffer, an index buffer and a program.
struct VertexArrayObject {
    id: gl::types::GLuint,
    destroyed: bool,
}

impl VertexArrayObject {
    /// Builds a new `VertexArrayObject`.
    ///
    /// The vertex buffer, index buffer and program must not outlive the
    /// VAO, and the VB & program attributes must not change.
    fn new(mut ctxt: &mut CommandContext, vertex_buffers: &[&VerticesSource],
           ib_id: gl::types::GLuint, program: &Program) -> VertexArrayObject
    {
        // checking the attributes types
        for vertex_buffer in vertex_buffers.iter() {
            let bindings = match vertex_buffer {
                &&VerticesSource::VertexBuffer(ref vertex_buffer, _, _) => {
                    vertex_buffer.get_bindings()
                },
                &&VerticesSource::PerInstanceBuffer(ref buffer) => {
                    buffer.get_bindings()
                },
            };

            for &(ref name, _, ty) in bindings.iter() {
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
            for vertex_buffer in vertex_buffers.iter() {
                let bindings = match vertex_buffer {
                    &&VerticesSource::VertexBuffer(ref vertex_buffer, _, _) => {
                        vertex_buffer.get_bindings()
                    },
                    &&VerticesSource::PerInstanceBuffer(ref buffer) => {
                        buffer.get_bindings()
                    },
                };

                if bindings.iter().find(|&&(ref n, _, _)| n == name).is_some() {
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("The program attribute `{}` is missing in the vertex bindings", name);
            }
        };

        // TODO: check for collisions between the vertices sources

        // building the values that will be sent to the other thread
        let data = vertex_buffers.iter().map(|vertex_buffer| {
            match vertex_buffer {
                &&VerticesSource::VertexBuffer(ref vertex_buffer, _, _) => {
                    (
                        GlObject::get_id(*vertex_buffer),
                        vertex_buffer.get_bindings().clone(),
                        vertex_buffer.get_elements_size(),
                        0 as u32
                    )
                },
                &&VerticesSource::PerInstanceBuffer(ref buffer) => {
                    (
                        GlObject::get_id(*buffer),
                        buffer.get_bindings().clone(),
                        buffer.get_elements_size(),
                        1 as u32
                    )
                },
            }
        }).collect::<Vec<_>>();

        let id = unsafe {
            // building the VAO
            let mut id = mem::uninitialized();
            if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) ||
                ctxt.version >= &context::GlVersion(Api::GlEs, 3, 0) ||
                ctxt.extensions.gl_arb_vertex_array_object
            {
                ctxt.gl.GenVertexArrays(1, &mut id);
            } else if ctxt.extensions.gl_oes_vertex_array_object {
                ctxt.gl.GenVertexArraysOES(1, &mut id);
            } else if ctxt.extensions.gl_apple_vertex_array_object {
                ctxt.gl.GenVertexArraysAPPLE(1, &mut id);
            } else {
                unreachable!();
            }

            // we don't use DSA as we're going to make multiple calls for this VAO
            // and we're likely going to use the VAO right after it's been created
            bind_vao(&mut ctxt, id);

            // binding index buffer
            // the ELEMENT_ARRAY_BUFFER is part of the state of the VAO
            // TODO: use a proper function
            if ctxt.version >= &context::GlVersion(Api::Gl, 1, 5) ||
                ctxt.version >= &context::GlVersion(Api::GlEs, 2, 0)
            {
                ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib_id);
            } else if ctxt.extensions.gl_arb_vertex_buffer_object {
                ctxt.gl.BindBufferARB(gl::ELEMENT_ARRAY_BUFFER_ARB, ib_id);
            } else {
                unreachable!();
            }

            for (vertex_buffer, bindings, vb_elementssize, divisor) in data.into_iter() {
                // binding vertex buffer because glVertexAttribPointer uses the current
                // array buffer
                // TODO: use a proper function
                if ctxt.state.array_buffer_binding != vertex_buffer {
                    if ctxt.version >= &context::GlVersion(Api::Gl, 1, 5) ||
                        ctxt.version >= &context::GlVersion(Api::GlEs, 2, 0)
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
                for (name, offset, ty) in bindings.into_iter() {
                    let (data_type, elements_count) = vertex_binding_type_to_gl(ty);

                    let attribute = match program.get_attribute(Borrow::<str>::borrow(&name)) {
                        Some(a) => a,
                        None => continue
                    };

                    let (attribute_ty, _) = vertex_binding_type_to_gl(attribute.ty);

                    if attribute.location != -1 {
                        match attribute_ty {
                            gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT |
                            gl::INT | gl::UNSIGNED_INT =>
                                ctxt.gl.VertexAttribIPointer(attribute.location as u32,
                                    elements_count as gl::types::GLint, data_type,
                                    vb_elementssize as i32, offset as *const libc::c_void),

                            gl::DOUBLE | gl::DOUBLE_VEC2 | gl::DOUBLE_VEC3 | gl::DOUBLE_VEC4 |
                            gl::DOUBLE_MAT2 | gl::DOUBLE_MAT3 | gl::DOUBLE_MAT4 |
                            gl::DOUBLE_MAT2x3 | gl::DOUBLE_MAT2x4 | gl::DOUBLE_MAT3x2 |
                            gl::DOUBLE_MAT3x4 | gl::DOUBLE_MAT4x2 | gl::DOUBLE_MAT4x3 =>
                                ctxt.gl.VertexAttribLPointer(attribute.location as u32,
                                    elements_count as gl::types::GLint, data_type,
                                    vb_elementssize as i32, offset as *const libc::c_void),

                            _ => ctxt.gl.VertexAttribPointer(attribute.location as u32,
                                    elements_count as gl::types::GLint, data_type, 0,
                                    vb_elementssize as i32, offset as *const libc::c_void)
                        }

                        if divisor != 0 {
                            ctxt.gl.VertexAttribDivisor(attribute.location as u32, divisor);
                        }

                        ctxt.gl.EnableVertexAttribArray(attribute.location as u32);
                    }
                }
            }

            id
        };

        VertexArrayObject {
            id: id,
            destroyed: false,
        }
    }

    fn destroy(mut self, mut ctxt: &mut CommandContext) {
        self.destroyed = true;

        unsafe {
            // unbinding
            if ctxt.state.vertex_array == self.id {
                if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) ||
                    ctxt.version >= &context::GlVersion(Api::GlEs, 3, 0) ||
                    ctxt.extensions.gl_arb_vertex_array_object
                {
                    ctxt.gl.BindVertexArray(0);
                } else if ctxt.extensions.gl_oes_vertex_array_object {
                    ctxt.gl.BindVertexArrayOES(0);
                } else if ctxt.extensions.gl_apple_vertex_array_object {
                    ctxt.gl.BindVertexArrayAPPLE(0);
                } else {
                    unreachable!();
                }

                ctxt.state.vertex_array = 0;
            }

            // deleting
            if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) ||
                ctxt.version >= &context::GlVersion(Api::GlEs, 3, 0) ||
                ctxt.extensions.gl_arb_vertex_array_object
            {
                ctxt.gl.DeleteVertexArrays(1, [ self.id ].as_ptr());
            } else if ctxt.extensions.gl_oes_vertex_array_object {
                ctxt.gl.DeleteVertexArraysOES(1, [ self.id ].as_ptr());
            } else if ctxt.extensions.gl_apple_vertex_array_object {
                ctxt.gl.DeleteVertexArraysAPPLE(1, [ self.id ].as_ptr());
            } else {
                unreachable!();
            }
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        assert!(self.destroyed);
    }
}

impl GlObject for VertexArrayObject {
    type Id = gl::types::GLuint;

    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

fn vertex_binding_type_to_gl(ty: AttributeType) -> (gl::types::GLenum, gl::types::GLint) {
    match ty {
        AttributeType::I8 => (gl::BYTE, 1),
        AttributeType::I8I8 => (gl::BYTE, 2),
        AttributeType::I8I8I8 => (gl::BYTE, 3),
        AttributeType::I8I8I8I8 => (gl::BYTE, 4),
        AttributeType::U8 => (gl::UNSIGNED_BYTE, 1),
        AttributeType::U8U8 => (gl::UNSIGNED_BYTE, 2),
        AttributeType::U8U8U8 => (gl::UNSIGNED_BYTE, 3),
        AttributeType::U8U8U8U8 => (gl::UNSIGNED_BYTE, 4),
        AttributeType::I16 => (gl::SHORT, 1),
        AttributeType::I16I16 => (gl::SHORT, 2),
        AttributeType::I16I16I16 => (gl::SHORT, 3),
        AttributeType::I16I16I16I16 => (gl::SHORT, 4),
        AttributeType::U16 => (gl::UNSIGNED_SHORT, 1),
        AttributeType::U16U16 => (gl::UNSIGNED_SHORT, 2),
        AttributeType::U16U16U16 => (gl::UNSIGNED_SHORT, 3),
        AttributeType::U16U16U16U16 => (gl::UNSIGNED_SHORT, 4),
        AttributeType::I32 => (gl::INT, 1),
        AttributeType::I32I32 => (gl::INT, 2),
        AttributeType::I32I32I32 => (gl::INT, 3),
        AttributeType::I32I32I32I32 => (gl::INT, 4),
        AttributeType::U32 => (gl::UNSIGNED_INT, 1),
        AttributeType::U32U32 => (gl::UNSIGNED_INT, 2),
        AttributeType::U32U32U32 => (gl::UNSIGNED_INT, 3),
        AttributeType::U32U32U32U32 => (gl::UNSIGNED_INT, 4),
        AttributeType::F32 => (gl::FLOAT, 1),
        AttributeType::F32F32 => (gl::FLOAT, 2),
        AttributeType::F32F32F32 => (gl::FLOAT, 3),
        AttributeType::F32F32F32F32 => (gl::FLOAT, 4),
        AttributeType::F32x2x2 => (gl::FLOAT_MAT2, 1),
        AttributeType::F32x2x3 => (gl::FLOAT_MAT2x3, 1),
        AttributeType::F32x2x4 => (gl::FLOAT_MAT2x4, 1),
        AttributeType::F32x3x2 => (gl::FLOAT_MAT3x2, 1),
        AttributeType::F32x3x3 => (gl::FLOAT_MAT3, 1),
        AttributeType::F32x3x4 => (gl::FLOAT_MAT3x4, 1),
        AttributeType::F32x4x2 => (gl::FLOAT_MAT4x2, 1),
        AttributeType::F32x4x3 => (gl::FLOAT_MAT4x3, 1),
        AttributeType::F32x4x4 => (gl::FLOAT_MAT4, 1),
        AttributeType::F64 => (gl::DOUBLE, 1),
        AttributeType::F64F64 => (gl::DOUBLE, 2),
        AttributeType::F64F64F64 => (gl::DOUBLE, 3),
        AttributeType::F64F64F64F64 => (gl::DOUBLE, 4),
        AttributeType::F64x2x2 => (gl::DOUBLE_MAT2, 1),
        AttributeType::F64x2x3 => (gl::DOUBLE_MAT2x3, 1),
        AttributeType::F64x2x4 => (gl::DOUBLE_MAT2x4, 1),
        AttributeType::F64x3x2 => (gl::DOUBLE_MAT3x2, 1),
        AttributeType::F64x3x3 => (gl::DOUBLE_MAT3, 1),
        AttributeType::F64x3x4 => (gl::DOUBLE_MAT3x4, 1),
        AttributeType::F64x4x2 => (gl::DOUBLE_MAT4x2, 1),
        AttributeType::F64x4x3 => (gl::DOUBLE_MAT4x3, 1),
        AttributeType::F64x4x4 => (gl::DOUBLE_MAT4, 1),
    }
}

fn bind_vao(ctxt: &mut CommandContext, vao_id: gl::types::GLuint) {
    if ctxt.state.vertex_array != vao_id {
        if ctxt.version >= &context::GlVersion(Api::Gl, 3, 0) ||
            ctxt.version >= &context::GlVersion(Api::GlEs, 3, 0) ||
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
