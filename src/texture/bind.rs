use gl;

use DrawError;
use TextureExt;

use uniforms::SamplerBehavior;

use context::CommandContext;
use ContextExt;

use utils::bitsfield::Bitsfield;

use vertex::MultiVerticesSource;

use version::Version;
use version::Api;

pub fn start_bind() -> Binder {
    Binder {
        texture_bind_points: Bitsfield::new(),
    }
}

pub enum BindError {

}

impl From<BindError> for DrawError {
    fn from(_: BindError) -> DrawError {
        unreachable!()
    }
}

pub struct Binder {
    texture_bind_points: Bitsfield,
}

impl Binder {
    // TODO: the context should be inside the Binder
    // TODO: check whether the selected sampling is usable with this texture
    pub fn add<T>(&mut self, ctxt: &mut CommandContext, texture: &T,
                  sampler: Option<SamplerBehavior>) -> Result<gl::types::GLint, BindError>
                  where T: TextureExt
    {
        let sampler = if let Some(sampler) = sampler {
            Some(::sampler_object::get_sampler(ctxt, &sampler).unwrap())        // TODO: don't unwrap
        } else {
            None
        };

        let sampler = sampler.unwrap_or(0);

        // finding an appropriate texture unit
        let texture_unit =
            ctxt.state.texture_units
                .iter().enumerate()
                .find(|&(unit, content)| {
                    content.texture == texture.get_texture_id() && (content.sampler == sampler ||
                                                    !self.texture_bind_points.is_used(unit as u16))
                })
                .map(|(unit, _)| unit as u16)
                .or_else(|| {
                    if ctxt.state.texture_units.len() <
                        ctxt.capabilities.max_combined_texture_image_units as usize
                    {
                        Some(ctxt.state.texture_units.len() as u16)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    self.texture_bind_points.get_unused().expect("Not enough texture units available")
                });
        assert!((texture_unit as gl::types::GLint) <
                ctxt.capabilities.max_combined_texture_image_units);
        self.texture_bind_points.set_used(texture_unit);

        // updating the state of the texture unit
        if ctxt.state.texture_units.len() <= texture_unit as usize {
            for _ in (ctxt.state.texture_units.len() .. texture_unit as usize + 1) {
                ctxt.state.texture_units.push(Default::default());
            }
        }

        // TODO: do better
        if ctxt.state.texture_units[texture_unit as usize].texture != texture.get_texture_id() ||
           ctxt.state.texture_units[texture_unit as usize].sampler != sampler
        {
            // TODO: what if it's not supported?
            if ctxt.state.active_texture != texture_unit as gl::types::GLenum {
                unsafe { ctxt.gl.ActiveTexture(texture_unit as gl::types::GLenum + gl::TEXTURE0) };
                ctxt.state.active_texture = texture_unit as gl::types::GLenum;
            }

            texture.bind_to_current(ctxt);

            if ctxt.state.texture_units[texture_unit as usize].sampler != sampler {
                assert!(ctxt.version >= &Version(Api::Gl, 3, 3) ||
                        ctxt.extensions.gl_arb_sampler_objects);

                unsafe { ctxt.gl.BindSampler(texture_unit as gl::types::GLenum, sampler); }
                ctxt.state.texture_units[texture_unit as usize].sampler = sampler;
            }
        }

        // returning the texture unit
        Ok(texture_unit as gl::types::GLint)
    }
}
