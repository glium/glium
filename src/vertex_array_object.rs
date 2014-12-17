use std::sync::Arc;
use std::mem;

use program::Program;
use vertex_buffer::{mod, VertexBuffer};
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
        let (tx, rx) = channel();

        let bindings = vertex_buffer::get_bindings(vertex_buffer).clone();
        let vb_elementssize = vertex_buffer::get_elements_size(vertex_buffer);
        let vertex_buffer = GlObject::get_id(vertex_buffer);
        let attributes = ::program::get_attributes(program);

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
                for (name, vertex_buffer::VertexAttrib { offset, data_type, elements_count })
                    in bindings.into_iter()
                {
                    let attribute = match attributes.get(&name) {
                        Some(a) => a,
                        None => continue
                    };

                    // FIXME: the reflected attributes can be (GL_FLOAT_VEC2, 1) while the
                    //        vertex buffer can be (GL_FLOAT, 2) ; need to check these cases
                    /*if attribute.ty != data_type || attribute.size != elements_count as i32 {
                        panic!("The program attributes do not match the vertex format {} {}");
                    }*/

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
    let mut vaos = display.vertex_array_objects.lock();

    let ib_id = indices.to_indices_source_helper().index_buffer.map(|b| b.get_id()).unwrap_or(0);
    let vb_id = GlObject::get_id(vertex_buffer);
    let program_id = program.get_id();

    if let Some(value) = vaos.get(&(vb_id, ib_id, program_id)) {
        return value.id;
    }

    let new_vao = VertexArrayObject::new(display.clone(), vertex_buffer, ib_id, program);
    let new_vao_id = new_vao.id;
    vaos.insert((vb_id, ib_id, program_id), new_vao);
    new_vao_id
}
