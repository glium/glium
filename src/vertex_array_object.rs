use std::sync::Arc;
use std::mem;

use program::Program;
use vertex_buffer::{VertexBuffer, BindingType};
use {DisplayImpl, IndicesSource, GlObject};

use {libc, gl};

/// 
pub struct VertexArrayObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl VertexArrayObject {
    /// 
    fn new<T>(display: Arc<DisplayImpl>, vertex_buffer: &VertexBuffer<T>,
              ib_id: gl::types::GLuint, program: &Program) -> VertexArrayObject
    {
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

pub fn get_vertex_array_object<T, I>(display: &Arc<DisplayImpl>, vertex_buffer: &VertexBuffer<T>,
                                     indices: &I, program: &Program) -> gl::types::GLuint
                                     where I: IndicesSource
{
    let ib_id = indices.to_indices_source_helper().index_buffer.map(|b| b.get_id()).unwrap_or(0);
    let vb_id = GlObject::get_id(vertex_buffer);
    let program_id = program.get_id();

    if let Some(value) = display.vertex_array_objects.lock().get(&(vb_id, ib_id, program_id)) {
        return value.id;
    }

    // we create the new VAO without the mutex locked
    let new_vao = VertexArrayObject::new(display.clone(), vertex_buffer, ib_id, program);
    let new_vao_id = new_vao.id;
    display.vertex_array_objects.lock().insert((vb_id, ib_id, program_id), new_vao);
    new_vao_id
}

fn vertex_binding_type_to_gl(ty: BindingType) -> (gl::types::GLenum, gl::types::GLint) {
    match ty {
        BindingType::I8 => (gl::BYTE, 1),
        BindingType::I8I8 => (gl::BYTE, 2),
        BindingType::I8I8I8 => (gl::BYTE, 3),
        BindingType::I8I8I8I8 => (gl::BYTE, 4),
        BindingType::U8 => (gl::UNSIGNED_BYTE, 1),
        BindingType::U8U8 => (gl::UNSIGNED_BYTE, 2),
        BindingType::U8U8U8 => (gl::UNSIGNED_BYTE, 3),
        BindingType::U8U8U8U8 => (gl::UNSIGNED_BYTE, 4),
        BindingType::I16 => (gl::SHORT, 1),
        BindingType::I16I16 => (gl::SHORT, 2),
        BindingType::I16I16I16 => (gl::SHORT, 3),
        BindingType::I16I16I16I16 => (gl::SHORT, 4),
        BindingType::U16 => (gl::UNSIGNED_SHORT, 1),
        BindingType::U16U16 => (gl::UNSIGNED_SHORT, 2),
        BindingType::U16U16U16 => (gl::UNSIGNED_SHORT, 3),
        BindingType::U16U16U16U16 => (gl::UNSIGNED_SHORT, 4),
        BindingType::I32 => (gl::INT, 1),
        BindingType::I32I32 => (gl::INT, 2),
        BindingType::I32I32I32 => (gl::INT, 3),
        BindingType::I32I32I32I32 => (gl::INT, 4),
        BindingType::U32 => (gl::UNSIGNED_INT, 1),
        BindingType::U32U32 => (gl::UNSIGNED_INT, 2),
        BindingType::U32U32U32 => (gl::UNSIGNED_INT, 3),
        BindingType::U32U32U32U32 => (gl::UNSIGNED_INT, 4),
        BindingType::F32 => (gl::FLOAT, 1),
        BindingType::F32F32 => (gl::FLOAT, 2),
        BindingType::F32F32F32 => (gl::FLOAT, 3),
        BindingType::F32F32F32F32 => (gl::FLOAT, 4),
    }
}

fn vertex_type_matches(ty: BindingType, gl_ty: gl::types::GLenum,
                       gl_size: gl::types::GLint) -> bool
{
    match (ty, gl_ty, gl_size) {
        (BindingType::I8, gl::BYTE, 1) => true,
        (BindingType::I8I8, gl::BYTE, 2) => true,
        (BindingType::I8I8I8, gl::BYTE, 3) => true,
        (BindingType::I8I8I8I8, gl::BYTE, 4) => true,
        (BindingType::U8, gl::UNSIGNED_BYTE, 1) => true,
        (BindingType::U8U8, gl::UNSIGNED_BYTE, 2) => true,
        (BindingType::U8U8U8, gl::UNSIGNED_BYTE, 3) => true,
        (BindingType::U8U8U8U8, gl::UNSIGNED_BYTE, 4) => true,
        (BindingType::I16, gl::SHORT, 1) => true,
        (BindingType::I16I16, gl::SHORT, 2) => true,
        (BindingType::I16I16I16, gl::SHORT, 3) => true,
        (BindingType::I16I16I16I16, gl::SHORT, 4) => true,
        (BindingType::U16, gl::UNSIGNED_SHORT, 1) => true,
        (BindingType::U16U16, gl::UNSIGNED_SHORT, 2) => true,
        (BindingType::U16U16U16, gl::UNSIGNED_SHORT, 3) => true,
        (BindingType::U16U16U16U16, gl::UNSIGNED_SHORT, 4) => true,
        (BindingType::I32, gl::INT, 1) => true,
        (BindingType::I32I32, gl::INT, 2) => true,
        (BindingType::I32I32, gl::INT_VEC2, 1) => true,
        (BindingType::I32I32I32, gl::INT, 3) => true,
        (BindingType::I32I32I32, gl::INT_VEC3, 1) => true,
        (BindingType::I32I32I32I32, gl::INT, 4) => true,
        (BindingType::I32I32I32I32, gl::INT_VEC4, 1) => true,
        (BindingType::I32I32I32I32, gl::INT_VEC2, 2) => true,
        (BindingType::U32, gl::UNSIGNED_INT, 1) => true,
        (BindingType::U32U32, gl::UNSIGNED_INT, 2) => true,
        (BindingType::U32U32, gl::UNSIGNED_INT_VEC2, 1) => true,
        (BindingType::U32U32U32, gl::UNSIGNED_INT, 3) => true,
        (BindingType::U32U32U32, gl::UNSIGNED_INT_VEC3, 1) => true,
        (BindingType::U32U32U32U32, gl::UNSIGNED_INT, 4) => true,
        (BindingType::U32U32U32U32, gl::UNSIGNED_INT_VEC4, 1) => true,
        (BindingType::U32U32U32U32, gl::UNSIGNED_INT_VEC2, 2) => true,
        (BindingType::F32, gl::FLOAT, 1) => true,
        (BindingType::F32F32, gl::FLOAT, 2) => true,
        (BindingType::F32F32, gl::FLOAT_VEC2, 1) => true,
        (BindingType::F32F32F32, gl::FLOAT, 3) => true,
        (BindingType::F32F32F32, gl::FLOAT_VEC3, 1) => true,
        (BindingType::F32F32F32F32, gl::FLOAT, 4) => true,
        (BindingType::F32F32F32F32, gl::FLOAT_VEC4, 1) => true,
        (BindingType::F32F32F32F32, gl::FLOAT_VEC2, 2) => true,
        _ => false,
    }
}
