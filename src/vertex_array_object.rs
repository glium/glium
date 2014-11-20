use std::sync::Arc;
use std::mem;

use vertex_buffer::{mod, VertexBuffer};
use {DisplayImpl, GlObject};

use {libc, gl};

/// 
pub struct VertexArrayObject {
    display: Arc<DisplayImpl>,
    id: gl::types::GLuint,
}

impl VertexArrayObject {
    /// 
    fn new<T>(display: Arc<DisplayImpl>, vertex_buffer: &VertexBuffer<T>,
        program_id: gl::types::GLuint) -> VertexArrayObject
    {
        let (tx, rx) = channel();

        let bindings = vertex_buffer::get_bindings(vertex_buffer).clone();
        let vb_elementssize = vertex_buffer::get_elements_size(vertex_buffer);
        let vertex_buffer = GlObject::get_id(vertex_buffer);

        display.context.exec(proc(ctxt) {
            unsafe {
                let id: gl::types::GLuint = mem::uninitialized();
                ctxt.gl.GenVertexArrays(1, mem::transmute(&id));
                tx.send(id);

                ctxt.gl.BindVertexArray(id);
                ctxt.state.vertex_array = id;

                // binding vertex buffer
                if ctxt.state.array_buffer_binding != Some(vertex_buffer) {
                    ctxt.gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
                    ctxt.state.array_buffer_binding = Some(vertex_buffer);
                }

                // binding index buffer
                // TODO: not sure if this is necessary
                ctxt.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                ctxt.state.element_array_buffer_binding = None;

                // binding attributes
                for (name, vertex_buffer::VertexAttrib { offset, data_type, elements_count })
                    in bindings.into_iter()
                {
                    let loc = ctxt.gl.GetAttribLocation(program_id, name.to_c_str().unwrap());

                    if loc != -1 {
                        match data_type {
                            gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT |
                            gl::INT | gl::UNSIGNED_INT =>
                                ctxt.gl.VertexAttribIPointer(loc as u32,
                                    elements_count as gl::types::GLint, data_type,
                                    vb_elementssize as i32, offset as *const libc::c_void),

                            _ => ctxt.gl.VertexAttribPointer(loc as u32,
                                    elements_count as gl::types::GLint, data_type, 0,
                                    vb_elementssize as i32, offset as *const libc::c_void)
                        }
                        
                        ctxt.gl.EnableVertexAttribArray(loc as u32);
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
        self.display.context.exec(proc(ctxt) {
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

pub fn get_vertex_array_object<T>(display: &Arc<DisplayImpl>, vertex_buffer: &VertexBuffer<T>,
    program_id: gl::types::GLuint) -> gl::types::GLuint
{
    let mut vaos = display.vertex_array_objects.lock();

    let vb_id = GlObject::get_id(vertex_buffer);
    if let Some(value) = vaos.get(&(vb_id, program_id)) {
        return value.id;
    }

    let new_vao = VertexArrayObject::new(display.clone(), vertex_buffer, program_id);
    let new_vao_id = new_vao.id;
    vaos.insert((vb_id, program_id), new_vao);
    new_vao_id
}
