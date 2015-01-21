use Display;
use DrawError;

use std::sync::mpsc;
use uniforms::SamplerBehavior;

use gl;
use context;
use GlObject;
use ToGlEnum;

/// An OpenGL sampler object.
pub struct SamplerObject {
    display: Display,
    id: gl::types::GLuint,
}

impl SamplerObject {
    pub fn new(display: &Display, behavior: &SamplerBehavior) -> SamplerObject {
        let (tx, rx) = mpsc::channel();

        let behavior = behavior.clone();
        display.context.context.exec(move |: ctxt| {
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

            tx.send(sampler).unwrap();
        });

        SamplerObject {
            display: display.clone(),
            id: rx.recv().unwrap(),
        }
    }
}

impl GlObject for SamplerObject {
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for SamplerObject {
    fn drop(&mut self) {
        let id = self.id;
        self.display.context.context.exec(move |: ctxt| {
            unsafe {
                ctxt.gl.DeleteSamplers(1, [id].as_ptr());
            }
        });
    }
}

pub fn get_sampler(display: &Display, behavior: &SamplerBehavior)
                   -> Result<gl::types::GLuint, DrawError>
{
    if display.context.context.get_version() <= &context::GlVersion(3, 3) &&
        !display.context.context.get_extensions().gl_arb_sampler_objects
    {
        return Err(DrawError::SamplersNotSupported);
    }

    match display.context.samplers.lock().unwrap().get(behavior) {
        Some(obj) => return Ok(obj.get_id()),
        None => ()
    };

    let sampler = SamplerObject::new(display, behavior);
    let id = sampler.get_id();
    display.context.samplers.lock().unwrap().insert(behavior.clone(), sampler);
    Ok(id)
}
