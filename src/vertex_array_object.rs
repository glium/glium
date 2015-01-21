use std::sync::Arc;
use std::sync::mpsc::channel;
use std::mem;

use program::Program;
use index_buffer::IndicesSource;
use vertex::{VerticesSource, AttributeType};
use {DisplayImpl, GlObject};

use {libc, gl, context};

/// Stores informations about how to bind a vertex buffer, an index buffer and a program.
pub struct VertexArrayObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl VertexArrayObject {
    /// Builds a new `VertexArrayObject`.
    ///
    /// The vertex buffer, index buffer and program must not outlive the
    /// VAO, and the VB & program attributes must not change.
    fn new(display: Arc<DisplayImpl>, vertex_buffers: &[&VerticesSource],
           ib_id: gl::types::GLuint, program: &Program) -> VertexArrayObject
    {
        let attributes = ::program::get_attributes(program);

        // checking the attributes types
        for vertex_buffer in vertex_buffers.iter() {
            let bindings = match vertex_buffer {
                &&VerticesSource::VertexBuffer(ref vertex_buffer, _) => {
                    vertex_buffer.get_bindings()
                },
                &&VerticesSource::PerInstanceBuffer(ref buffer, _) => {
                    buffer.get_bindings()
                },
            };

            for &(ref name, _, ty) in bindings.iter() {
                let attribute = match attributes.get(name) {
                    Some(a) => a,
                    None => continue
                };

                if !vertex_type_matches(ty, attribute.ty, attribute.size) {
                    panic!("The program attribute `{}` does not match the vertex format", name);
                }
            }
        }

        // checking for missing attributes
        for (&ref name, _) in attributes.iter() {
            let mut found = false;
            for vertex_buffer in vertex_buffers.iter() {
                let bindings = match vertex_buffer {
                    &&VerticesSource::VertexBuffer(ref vertex_buffer, _) => {
                        vertex_buffer.get_bindings()
                    },
                    &&VerticesSource::PerInstanceBuffer(ref buffer, _) => {
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
                &&VerticesSource::VertexBuffer(ref vertex_buffer, _) => {
                    (
                        GlObject::get_id(*vertex_buffer),
                        vertex_buffer.get_bindings().clone(),
                        vertex_buffer.get_elements_size(),
                        0 as u32
                    )
                },
                &&VerticesSource::PerInstanceBuffer(ref buffer, _) => {
                    (
                        GlObject::get_id(*buffer),
                        buffer.get_bindings().clone(),
                        buffer.get_elements_size(),
                        1 as u32
                    )
                },
            }
        }).collect::<Vec<_>>();

        let (tx, rx) = channel();

        display.context.exec(move |: ctxt| {
            unsafe {
                // building the VAO
                let id: gl::types::GLuint = mem::uninitialized();
                if ctxt.version >= &context::GlVersion(3, 0) ||
                    ctxt.extensions.gl_arb_vertex_array_object
                {
                    ctxt.gl.GenVertexArrays(1, mem::transmute(&id));
                } else if ctxt.extensions.gl_apple_vertex_array_object {
                    ctxt.gl.GenVertexArraysAPPLE(1, mem::transmute(&id));
                } else {
                    unreachable!()
                }
                tx.send(id).unwrap();

                // we don't use DSA as we're going to make multiple calls for this VAO
                // and we're likely going to use the VAO right after it's been created
                if ctxt.version >= &context::GlVersion(3, 0) ||
                    ctxt.extensions.gl_arb_vertex_array_object
                {
                    ctxt.gl.BindVertexArray(id);
                } else if ctxt.extensions.gl_apple_vertex_array_object {
                    ctxt.gl.BindVertexArrayAPPLE(id);
                } else {
                    unreachable!()
                }
                ctxt.state.vertex_array = id;

                // binding index buffer
                // the ELEMENT_ARRAY_BUFFER is part of the state of the VAO
                ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib_id);

                for (vertex_buffer, bindings, vb_elementssize, divisor) in data.into_iter() {
                    // binding vertex buffer because glVertexAttribPointer uses the current
                    // array buffer
                    if ctxt.state.array_buffer_binding != vertex_buffer {
                        ctxt.gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
                        ctxt.state.array_buffer_binding = vertex_buffer;
                    }

                    // binding attributes
                    for (name, offset, ty) in bindings.into_iter() {
                        let (data_type, elements_count) = vertex_binding_type_to_gl(ty);

                        let attribute = match attributes.get(&name) {
                            Some(a) => a,
                            None => continue
                        };

                        if attribute.location != -1 {
                            match data_type {
                                gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT |
                                gl::INT | gl::UNSIGNED_INT =>
                                    ctxt.gl.VertexAttribIPointer(attribute.location as u32,
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
            }
        });

        VertexArrayObject {
            display: display,
            id: rx.recv().unwrap(),
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.exec(move |: ctxt| {
            unsafe {
                // unbinding
                if ctxt.state.vertex_array == id {
                    if ctxt.version >= &context::GlVersion(3, 0) ||
                        ctxt.extensions.gl_arb_vertex_array_object
                    {
                        ctxt.gl.BindVertexArray(0);
                    } else if ctxt.extensions.gl_apple_vertex_array_object {
                        ctxt.gl.BindVertexArrayAPPLE(0);
                    } else {
                        unreachable!()
                    }

                    ctxt.state.vertex_array = 0;
                }

                // deleting
                if ctxt.version >= &context::GlVersion(3, 0) ||
                    ctxt.extensions.gl_arb_vertex_array_object
                {
                    ctxt.gl.DeleteVertexArrays(1, [ id ].as_ptr());
                } else if ctxt.extensions.gl_apple_vertex_array_object {
                    ctxt.gl.DeleteVertexArraysAPPLE(1, [ id ].as_ptr());
                } else {
                    unreachable!()
                }
            }
        });
    }
}

impl GlObject for VertexArrayObject {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

/// Obtains the id of the VAO corresponding to the vertex buffer, index buffer and program
/// passed as parameters. Creates a new VAO if no existing one matches these.
pub fn get_vertex_array_object<I>(display: &Arc<DisplayImpl>, vertex_buffers: &[&VerticesSource],
                                  indices: &IndicesSource<I>, program: &Program)
                                  -> gl::types::GLuint where I: ::index_buffer::Index
{
    let ib_id = match indices {
        &IndicesSource::Buffer { .. } => 0,
        &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_id()
    };

    let buffers_list = {
        let mut buffers_list = Vec::with_capacity(1 + vertex_buffers.len());
        if ib_id != 0 {
            buffers_list.push(ib_id);
        }
        for vertex_buffer in vertex_buffers.iter() {
            buffers_list.push(match vertex_buffer {
                &&VerticesSource::VertexBuffer(ref vb, _) => vb.get_id(),
                &&VerticesSource::PerInstanceBuffer(ref buf, _) => buf.get_id(),
            });
        }
        buffers_list.sort();
        buffers_list
    };

    let program_id = program.get_id();

    if let Some(value) = display.vertex_array_objects.lock().unwrap()
                                .get(&(buffers_list.clone(), program_id)) {
        return value.id;
    }

    // we create the new VAO without the mutex locked
    let new_vao = VertexArrayObject::new(display.clone(), vertex_buffers, ib_id, program);
    let new_vao_id = new_vao.id;
    display.vertex_array_objects.lock().unwrap().insert((buffers_list, program_id), new_vao);
    new_vao_id
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
    }
}

fn vertex_type_matches(ty: AttributeType, gl_ty: gl::types::GLenum,
                       gl_size: gl::types::GLint) -> bool
{
    match (ty, gl_ty, gl_size) {
        (AttributeType::I8, gl::BYTE, 1) => true,
        (AttributeType::I8I8, gl::BYTE, 2) => true,
        (AttributeType::I8I8I8, gl::BYTE, 3) => true,
        (AttributeType::I8I8I8I8, gl::BYTE, 4) => true,
        (AttributeType::U8, gl::UNSIGNED_BYTE, 1) => true,
        (AttributeType::U8U8, gl::UNSIGNED_BYTE, 2) => true,
        (AttributeType::U8U8U8, gl::UNSIGNED_BYTE, 3) => true,
        (AttributeType::U8U8U8U8, gl::UNSIGNED_BYTE, 4) => true,
        (AttributeType::I16, gl::SHORT, 1) => true,
        (AttributeType::I16I16, gl::SHORT, 2) => true,
        (AttributeType::I16I16I16, gl::SHORT, 3) => true,
        (AttributeType::I16I16I16I16, gl::SHORT, 4) => true,
        (AttributeType::U16, gl::UNSIGNED_SHORT, 1) => true,
        (AttributeType::U16U16, gl::UNSIGNED_SHORT, 2) => true,
        (AttributeType::U16U16U16, gl::UNSIGNED_SHORT, 3) => true,
        (AttributeType::U16U16U16U16, gl::UNSIGNED_SHORT, 4) => true,
        (AttributeType::I32, gl::INT, 1) => true,
        (AttributeType::I32I32, gl::INT, 2) => true,
        (AttributeType::I32I32, gl::INT_VEC2, 1) => true,
        (AttributeType::I32I32I32, gl::INT, 3) => true,
        (AttributeType::I32I32I32, gl::INT_VEC3, 1) => true,
        (AttributeType::I32I32I32I32, gl::INT, 4) => true,
        (AttributeType::I32I32I32I32, gl::INT_VEC4, 1) => true,
        (AttributeType::I32I32I32I32, gl::INT_VEC2, 2) => true,
        (AttributeType::U32, gl::UNSIGNED_INT, 1) => true,
        (AttributeType::U32U32, gl::UNSIGNED_INT, 2) => true,
        (AttributeType::U32U32, gl::UNSIGNED_INT_VEC2, 1) => true,
        (AttributeType::U32U32U32, gl::UNSIGNED_INT, 3) => true,
        (AttributeType::U32U32U32, gl::UNSIGNED_INT_VEC3, 1) => true,
        (AttributeType::U32U32U32U32, gl::UNSIGNED_INT, 4) => true,
        (AttributeType::U32U32U32U32, gl::UNSIGNED_INT_VEC4, 1) => true,
        (AttributeType::U32U32U32U32, gl::UNSIGNED_INT_VEC2, 2) => true,
        (AttributeType::F32, gl::FLOAT, 1) => true,
        (AttributeType::F32F32, gl::FLOAT, 2) => true,
        (AttributeType::F32F32, gl::FLOAT_VEC2, 1) => true,
        (AttributeType::F32F32F32, gl::FLOAT, 3) => true,
        (AttributeType::F32F32F32, gl::FLOAT_VEC3, 1) => true,
        (AttributeType::F32F32F32F32, gl::FLOAT, 4) => true,
        (AttributeType::F32F32F32F32, gl::FLOAT_VEC4, 1) => true,
        (AttributeType::F32F32F32F32, gl::FLOAT_VEC2, 2) => true,
        _ => false,
    }
}
