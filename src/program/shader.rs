use gl;
use version::Version;
use version::Api;

use CapabilitiesSource;
use backend::Facade;
use context::Context;
use ContextExt;

use std::{ffi, ptr};
use std::rc::Rc;

use GlObject;
use Handle;

use program::ProgramCreationError;

/// A single, compiled but unlinked, shader.
pub struct Shader {
    context: Rc<Context>,
    id: Handle,
}

impl GlObject for Shader {
    type Id = Handle;

    #[inline]
    fn get_id(&self) -> Handle {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let ctxt = self.context.make_current();

        unsafe {
            match self.id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                            ctxt.version >= &Version(Api::GlEs, 2, 0));
                    ctxt.gl.DeleteShader(id);
                },
                Handle::Handle(id) => {
                    assert!(ctxt.extensions.gl_arb_shader_objects);
                    ctxt.gl.DeleteObjectARB(id);
                }
            }
        }
    }
}

/// Builds an individual shader.
pub fn build_shader<F: ?Sized>(facade: &F, shader_type: gl::types::GLenum, source_code: &str)
                       -> Result<Shader, ProgramCreationError> where F: Facade
{
    unsafe {
        let mut ctxt = facade.get_context().make_current();

        if ctxt.capabilities.supported_glsl_versions.is_empty() {
            return Err(ProgramCreationError::CompilationNotSupported);
        }

        if !check_shader_type_compatibility(&mut ctxt, shader_type) {
            return Err(ProgramCreationError::ShaderTypeNotSupported);
        }

        let source_code = ffi::CString::new(source_code.as_bytes()).unwrap();

        let id = if ctxt.version >= &Version(Api::Gl, 2, 0) ||
                    ctxt.version >= &Version(Api::GlEs, 2, 0)
        {
            Handle::Id(ctxt.gl.CreateShader(shader_type))
        } else if ctxt.extensions.gl_arb_shader_objects {
            Handle::Handle(ctxt.gl.CreateShaderObjectARB(shader_type))
        } else {
            unreachable!()
        };

        if id == Handle::Id(0) || id == Handle::Handle(0 as gl::types::GLhandleARB) {
            return Err(ProgramCreationError::ShaderTypeNotSupported);
        }

        match id {
            Handle::Id(id) => {
                assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                        ctxt.version >= &Version(Api::GlEs, 2, 0));
                ctxt.gl.ShaderSource(id, 1, [ source_code.as_ptr() ].as_ptr(), ptr::null());
            },
            Handle::Handle(id) => {
                assert!(ctxt.extensions.gl_arb_shader_objects);
                ctxt.gl.ShaderSourceARB(id, 1, [ source_code.as_ptr() ].as_ptr(), ptr::null());
            }
        }

        // compiling
        {
            ctxt.report_debug_output_errors.set(false);

            match id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0)||
                            ctxt.version >= &Version(Api::GlEs, 2, 0));
                    ctxt.gl.CompileShader(id);
                },
                Handle::Handle(id) => {
                    assert!(ctxt.extensions.gl_arb_shader_objects);
                    ctxt.gl.CompileShaderARB(id);
                }
            }

            ctxt.report_debug_output_errors.set(true);
        }

        // checking compilation success by reading a flag on the shader
        let compilation_success = {
            let mut compilation_success: gl::types::GLint = 0;
            match id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                            ctxt.version >= &Version(Api::GlEs, 2, 0));
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

        if compilation_success == 1 {
            Ok(Shader {
                context: facade.get_context().clone(),
                id: id
            })

        } else {
            // compilation error
            let mut error_log_size: gl::types::GLint = 0;

            match id {
                Handle::Id(id) => {
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                            ctxt.version >= &Version(Api::GlEs, 2, 0));
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
                    assert!(ctxt.version >= &Version(Api::Gl, 2, 0) ||
                            ctxt.version >= &Version(Api::GlEs, 2, 0));
                    ctxt.gl.GetShaderInfoLog(id, error_log_size, &mut error_log_size,
                                             error_log.as_mut_ptr() as *mut gl::types::GLchar);
                },
                Handle::Handle(id) => {
                    assert!(ctxt.extensions.gl_arb_shader_objects);
                    ctxt.gl.GetInfoLogARB(id, error_log_size, &mut error_log_size,
                                          error_log.as_mut_ptr() as *mut gl::types::GLchar);
                }
            }

            error_log.set_len(error_log_size as usize);

            match String::from_utf8(error_log) {
                Ok(msg) => Err(ProgramCreationError::CompilationError(msg)),
                Err(_) => Err(
                    ProgramCreationError::CompilationError("Could not convert the log \
                                                            message to UTF-8".to_owned())
                ),
            }
        }
    }
}

pub fn check_shader_type_compatibility<C: ?Sized>(ctxt: &C, shader_type: gl::types::GLenum)
                                          -> bool where C: CapabilitiesSource
{
    match shader_type {
        gl::VERTEX_SHADER | gl::FRAGMENT_SHADER => (),
        gl::GEOMETRY_SHADER => {
            if !(ctxt.get_version() >= &Version(Api::Gl, 3, 2))
                && !(ctxt.get_version() >= &Version(Api::GlEs, 3, 2))
                && !ctxt.get_extensions().gl_arb_geometry_shader4
                && !ctxt.get_extensions().gl_ext_geometry_shader4
                && !ctxt.get_extensions().gl_ext_geometry_shader
                && !ctxt.get_extensions().gl_oes_geometry_shader
            {
                return false;
            }
        },
        gl::TESS_CONTROL_SHADER | gl::TESS_EVALUATION_SHADER => {
            if !(ctxt.get_version() >= &Version(Api::Gl, 4, 0))
                && !(ctxt.get_version() >= &Version(Api::GlEs, 3, 2))
                && !ctxt.get_extensions().gl_arb_tessellation_shader
                && !ctxt.get_extensions().gl_oes_tessellation_shader
            {
                return false;
            }
        },
        gl::COMPUTE_SHADER => {
            if !(ctxt.get_version() >= &Version(Api::Gl, 4, 3))
                && !(ctxt.get_version() >= &Version(Api::GlEs, 3, 1))
                && !ctxt.get_extensions().gl_arb_compute_shader
            {
                return false;
            }
        },
        _ => unreachable!()
    };

    true
}
