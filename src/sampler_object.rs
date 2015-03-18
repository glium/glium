use Display;
use DrawError;

use uniforms::SamplerBehavior;

use gl;
use context::{self, GlVersion};
use version::Api;
use GlObject;
use ToGlEnum;

/// An OpenGL sampler object.
pub struct SamplerObject {
    display: Display,
    id: gl::types::GLuint,
}

impl SamplerObject {
    /// Builds a new sampler object.
    pub fn new(display: &Display, behavior: &SamplerBehavior) -> SamplerObject {
        // making sure that the backend supports samplers
        assert!(display.context.context.get_version() >= &GlVersion(Api::Gl, 3, 2) ||
                display.context.context.get_extensions().gl_arb_sampler_objects);

        let ctxt = display.context.context.make_current();

        let sampler = unsafe {
            use std::mem;
            let mut sampler: gl::types::GLuint = mem::uninitialized();
            ctxt.gl.GenSamplers(1, &mut sampler);
            sampler
        };

        unsafe {
            ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_WRAP_S,
                                      behavior.wrap_function.0.to_glenum() as gl::types::GLint);
            ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_WRAP_T,
                                      behavior.wrap_function.1.to_glenum() as gl::types::GLint);
            ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_WRAP_R,
                                      behavior.wrap_function.2.to_glenum() as gl::types::GLint);
            ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_MIN_FILTER,
                                      behavior.minify_filter.to_glenum() as gl::types::GLint);
            ctxt.gl.SamplerParameteri(sampler, gl::TEXTURE_MAG_FILTER,
                                      behavior.magnify_filter.to_glenum() as gl::types::GLint);

            if let Some(max_value) = ctxt.capabilities.max_texture_max_anisotropy {
                let value = if behavior.max_anisotropy as f32 > max_value {
                    max_value
                } else {
                    behavior.max_anisotropy as f32
                };

                ctxt.gl.SamplerParameterf(sampler, gl::TEXTURE_MAX_ANISOTROPY_EXT, value);
            }
        }

        SamplerObject {
            display: display.clone(),
            id: sampler,
        }
    }
}

impl GlObject for SamplerObject {
    type Id = gl::types::GLuint;

    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for SamplerObject {
    fn drop(&mut self) {
        let ctxt = self.display.context.context.make_current();
        unsafe { ctxt.gl.DeleteSamplers(1, [self.id].as_ptr()); }
    }
}

/// Returns the sampler corresponding to the given behavior, or a draw error if
/// samplers are not supported.
pub fn get_sampler(display: &Display, behavior: &SamplerBehavior)
                   -> Result<gl::types::GLuint, DrawError>
{
    // checking for compatibility
    if display.context.context.get_version() < &context::GlVersion(Api::Gl, 3, 2) &&
        !display.context.context.get_extensions().gl_arb_sampler_objects
    {
        return Err(DrawError::SamplersNotSupported);
    }

    // looking for an existing sampler
    match display.context.context.samplers.borrow_mut().get(behavior) {
        Some(obj) => return Ok(obj.get_id()),
        None => ()
    };

    // builds a new sampler
    let sampler = SamplerObject::new(display, behavior);
    let id = sampler.get_id();
    display.context.context.samplers.borrow_mut().insert(behavior.clone(), sampler);
    Ok(id)
}
