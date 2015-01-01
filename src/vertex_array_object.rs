use std::sync::Arc;
use std::mem;

use program::Program;
use index_buffer::IndicesSource;
use vertex_buffer::{VerticesSource, AttributeType};
use {DisplayImpl, GlObject};

use {libc, gl};

/// 
pub struct VertexArrayObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl VertexArrayObject {
    /// 
    fn new(display: Arc<DisplayImpl>, vertex_buffer: VerticesSource,
           ib_id: gl::types::GLuint, program: &Program) -> VertexArrayObject
    {
        let VerticesSource::VertexBuffer(vertex_buffer) = vertex_buffer;
        let bindings = vertex_buffer.get_bindings().clone();
        let vb_elementssize = vertex_buffer.get_elements_size();
        let vertex_buffer = GlObject::get_id(vertex_buffer);
        let attributes = ::program::get_attributes(program);

        // checking the attributes types
        for &(ref name, _, ty) in bindings.iter() {
            let attribute = match attributes.get(name) {
                Some(a) => a,
                None => continue
            };

            if !vertex_type_matches(ty, attribute.ty, attribute.size) {
                panic!("The program attribute `{}` does not match the vertex format", name);
            }
        }

        // checking for missing attributes
        for (&ref name, _) in attributes.iter() {
            if bindings.iter().find(|&&(ref n, _, _)| n == name).is_none() {
                panic!("The program attribute `{}` is missing in the vertex bindings", name);
            }
        };

        let (tx, rx) = channel();

        display.context.exec(move |: ctxt| {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenVertexArrays(1, mem::transmute(&id));
                tx.send(id);

                ctxt.gl.BindVertexArray(id);
                ctxt.state.vertex_array = id;

                // binding vertex buffer
                if ctxt.state.array_buffer_binding != vertex_buffer {
                    ctxt.gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
                    ctxt.state.array_buffer_binding = vertex_buffer;
                }

                // binding index buffer
                ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib_id);

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
                        
                        ctxt.gl.EnableVertexAttribArray(attribute.location as u32);
                    }
                }
            }
        });

        VertexArrayObject {
            display: display,
            id: rx.recv(),
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
                    ctxt.gl.BindVertexArray(0);
                    ctxt.state.vertex_array = 0;
                }

                // deleting
                ctxt.gl.DeleteVertexArrays(1, [ id ].as_ptr());
            }
        });
    }
}

impl GlObject for VertexArrayObject {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

pub fn get_vertex_array_object<I>(display: &Arc<DisplayImpl>, vertex_buffer: VerticesSource,
                                  indices: &IndicesSource<I>, program: &Program)
                                  -> gl::types::GLuint where I: ::index_buffer::Index
{
    let ib_id = match indices {
        &IndicesSource::Buffer { .. } => 0,
        &IndicesSource::IndexBuffer { ref buffer, .. } => buffer.get_id()
    };

    let vb_id = match vertex_buffer {
        VerticesSource::VertexBuffer(vb) => vb.get_id(),
    };

    let program_id = program.get_id();

    if let Some(value) = display.vertex_array_objects.lock().unwrap()
                                .get(&(vb_id, ib_id, program_id)) {
        return value.id;
    }

    // we create the new VAO without the mutex locked
    let new_vao = VertexArrayObject::new(display.clone(), vertex_buffer.clone(), ib_id, program);
    let new_vao_id = new_vao.id;
    display.vertex_array_objects.lock().unwrap().insert((vb_id, ib_id, program_id), new_vao);
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
