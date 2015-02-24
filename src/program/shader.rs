use gl;
use context::CommandContext;
use context::GlVersion;
use version::Api;

use std::{ffi, mem, ptr};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;

use Display;
use GlObject;
use Handle;

use program::COMPILER_GLOBAL_LOCK;
use program::ProgramCreationError;

/// A single, compiled but unlinked, shader.
pub struct Shader {
    display: Display,
    id: Handle,
}

impl GlObject for Shader {
    type Id = Handle;

    fn get_id(&self) -> Handle {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let id = self.id.clone();
        self.display.context.context.exec(move |ctxt| {
            unsafe {
                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.DeleteShader(id);
                    },
                    Handle::Handle(id) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.DeleteObjectARB(id);
                    }
                }
            }
        });
    }
}

/// Builds an individual shader.
pub fn build_shader(display: &Display, shader_type: gl::types::GLenum, source_code: &str)
                    -> Result<Shader, ProgramCreationError>
{
    let source_code = ffi::CString::from_slice(source_code.as_bytes());

    let (tx, rx) = channel();
    display.context.context.exec(move |mut ctxt| {
        unsafe {
            match check_shader_type_compatibility(&mut ctxt, shader_type) {
                Ok(_) => {},
                Err(e) => {
                    tx.send(Err(e));
                    return;
                }
            }

            let id = if ctxt.version >= &GlVersion(Api::Gl, 2, 0) {
                Handle::Id(ctxt.gl.CreateShader(shader_type))
            } else if ctxt.extensions.gl_arb_shader_objects {
                Handle::Handle(ctxt.gl.CreateShaderObjectARB(shader_type))
            } else {
                unreachable!()
            };

            if id == Handle::Id(0) || id == Handle::Handle(0 as gl::types::GLhandleARB) {
                tx.send(Err(ProgramCreationError::ShaderTypeNotSupported)).ok();
                return;
            }

            match id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                    ctxt.gl.ShaderSource(id, 1, [ source_code.as_ptr() ].as_ptr(), ptr::null());
                },
                Handle::Handle(id) => {
                    assert!(ctxt.extensions.gl_arb_shader_objects);
                    ctxt.gl.ShaderSourceARB(id, 1, [ source_code.as_ptr() ].as_ptr(), ptr::null());
                }
            }

            // compiling
            {
                let _lock = COMPILER_GLOBAL_LOCK.lock();

                ctxt.shared_debug_output.report_errors.store(false, Ordering::Relaxed);

                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.CompileShader(id);
                    },
                    Handle::Handle(id) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.CompileShaderARB(id);
                    }
                }

                ctxt.shared_debug_output.report_errors.store(true, Ordering::Relaxed);
            }

            // checking compilation success
            let compilation_success = {
                let mut compilation_success: gl::types::GLint = mem::uninitialized();
                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut compilation_success);
                    },
                    Handle::Handle(id) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.GetObjectParameterivARB(id, gl::OBJECT_COMPILE_STATUS_ARB,
                                                        &mut compilation_success);
                    }
                }
                compilation_success
            };

            if compilation_success == 0 {
                // compilation error
                let mut error_log_size: gl::types::GLint = mem::uninitialized();

                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size);
                    },
                    Handle::Handle(id) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.GetObjectParameterivARB(id, gl::OBJECT_INFO_LOG_LENGTH_ARB,
                                                        &mut error_log_size);
                    }
                }

                let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);

                match id {
                    Handle::Id(id) => {
                        assert!(ctxt.version >= &GlVersion(Api::Gl, 2, 0));
                        ctxt.gl.GetShaderInfoLog(id, error_log_size, &mut error_log_size,
                                                 error_log.as_mut_slice().as_mut_ptr()
                                                   as *mut gl::types::GLchar);
                    },
                    Handle::Handle(id) => {
                        assert!(ctxt.extensions.gl_arb_shader_objects);
                        ctxt.gl.GetInfoLogARB(id, error_log_size, &mut error_log_size,
                                              error_log.as_mut_slice().as_mut_ptr()
                                                as *mut gl::types::GLchar);
                    }
                }

                error_log.set_len(error_log_size as usize);

                let msg = String::from_utf8(error_log).unwrap();
                tx.send(Err(ProgramCreationError::CompilationError(msg))).ok();
                return;
            }

            tx.send(Ok(id)).unwrap();
        }
    });

    rx.recv().unwrap().map(|id| {
        Shader {
            display: display.clone(),
            id: id
        }
    })
}

fn check_shader_type_compatibility(ctxt: &mut CommandContext, shader_type: gl::types::GLenum)
                                   -> Result<(), ProgramCreationError>
{
    match shader_type {
        gl::VERTEX_SHADER | gl::FRAGMENT_SHADER => (),
        gl::GEOMETRY_SHADER => {
            if ctxt.version < &GlVersion(Api::Gl, 3, 0) && !ctxt.extensions.gl_arb_geometry_shader4
               && !ctxt.extensions.gl_ext_geometry_shader4
            {
                return Err(ProgramCreationError::ShaderTypeNotSupported);
            }
        },
        gl::TESS_CONTROL_SHADER | gl::TESS_EVALUATION_SHADER => {
            if ctxt.version < &GlVersion(Api::Gl, 4, 0) &&
               !ctxt.extensions.gl_arb_tessellation_shader 
            {
                return Err(ProgramCreationError::ShaderTypeNotSupported);
            }
        },
        gl::COMPUTE_SHADER => {
            if ctxt.version < &GlVersion(Api::Gl, 4, 3) && !ctxt.extensions.gl_arb_compute_shader {
                return Err(ProgramCreationError::ShaderTypeNotSupported);
            }
        },
        _ => unreachable!()
    };

    Ok(())
}
