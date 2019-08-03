use context::CommandContext;
use version::Version;
use version::Api;
use gl;

use std::fmt;
use std::error::Error;

use texture::any::TextureAny;
use TextureExt;
use GlObject;

/// Error that can happen when retrieving the internal format of a texture.
#[derive(Copy, Clone, Debug)]
pub enum GetFormatError {
    /// The backend doesn't support retrieving the internal format.
    NotSupported,
}

impl fmt::Display for GetFormatError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for GetFormatError {
    fn description(&self) -> &str {
        use self::GetFormatError::*;
        match *self {
            NotSupported =>
                "The backend doesn't support retrieving the internal format",
        }
    }
}

/// Internal format of a texture.
///
/// The actual format of a texture is not necessarily one of the predefined ones, so we have
/// to use a very generic description.
// TODO: change bits to be u16 for consistency with the rest of the library
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InternalFormat {
    /// The format has one component.
    OneComponent {
        /// Type of the first component of the format.
        ty1: InternalFormatType,
        /// Number of bits of the first component.
        bits1: usize,
    },

    /// The format has two components.
    TwoComponents {
        /// Type of the first component of the format.
        ty1: InternalFormatType,
        /// Number of bits of the first component.
        bits1: usize,
        /// Type of the second component.
        ty2: InternalFormatType,
        /// Number of bits of the second component.
        bits2: usize,
    },

    /// The format has three components.
    ThreeComponents {
        /// Type of the first component of the format.
        ty1: InternalFormatType,
        /// Number of bits of the first component.
        bits1: usize,
        /// Type of the second component.
        ty2: InternalFormatType,
        /// Number of bits of the second component.
        bits2: usize,
        /// Type of the third component.
        ty3: InternalFormatType,
        /// Number of bits of the third component.
        bits3: usize,
    },

    /// The format has four components.
    FourComponents {
        /// Type of the first component of the format.
        ty1: InternalFormatType,
        /// Number of bits of the first component.
        bits1: usize,
        /// Type of the second component.
        ty2: InternalFormatType,
        /// Number of bits of the second component.
        bits2: usize,
        /// Type of the third component.
        ty3: InternalFormatType,
        /// Number of bits of the third component.
        bits3: usize,
        /// Type of the fourth component.
        ty4: InternalFormatType,
        /// Number of bits of the fourth component.
        bits4: usize,
    },
}

impl InternalFormat {
    /// Returns the total number of bits of this format.
    #[inline]
    pub fn get_total_bits(&self) -> usize {
        match self {
            &InternalFormat::OneComponent { bits1, .. } => bits1,
            &InternalFormat::TwoComponents { bits1, bits2, .. } => bits1 + bits2,
            &InternalFormat::ThreeComponents { bits1, bits2, bits3, .. } => bits1 + bits2 + bits3,
            &InternalFormat::FourComponents { bits1, bits2, bits3, bits4, .. } =>
                                                                    bits1 + bits2 + bits3 + bits4,
        }
    }
}

/// Format of a component of an internal format.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InternalFormatType {
    /// Floating point texture with signed components.
    SignedNormalized,
    /// Floating point texture with unsigned components.
    UnsignedNormalized,
    /// Floating point texture with floats.
    Float,
    /// Integral texture.
    Int,
    /// Unsigned texture.
    UnsignedInt,
}

impl InternalFormatType {
    /// Builds the `InternalFormatType` of the GLenum. Panics if the value is not recognized.
    #[inline]
    fn from_glenum(val: gl::types::GLenum) -> InternalFormatType {
        match val {
            gl::SIGNED_NORMALIZED => InternalFormatType::SignedNormalized,
            gl::UNSIGNED_NORMALIZED => InternalFormatType::UnsignedNormalized,
            gl::FLOAT => InternalFormatType::Float,
            gl::INT => InternalFormatType::Int,
            gl::UNSIGNED_INT => InternalFormatType::UnsignedInt,
            gl::NONE => unreachable!(),     // separating this case for easier debugging
            _ => unreachable!(),
        }
    }
}

/// Determines the format of a texture.
///
/// A `TextureAny` is guaranteed to have the same format for each mipmap.
pub fn get_format(ctxt: &mut CommandContext, texture: &TextureAny)
                  -> Result<InternalFormat, GetFormatError>
{
    if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.version >= &Version(Api::GlEs, 3, 0) {
        let (red_sz, red_ty, green_sz, green_ty, blue_sz, blue_ty,
             alpha_sz, alpha_ty, depth_sz, depth_ty) = unsafe
        {
            let mut red_sz = 0;
            let mut red_ty = 0;
            let mut green_sz = 0;
            let mut green_ty = 0;
            let mut blue_sz = 0;
            let mut blue_ty = 0;
            let mut alpha_sz = 0;
            let mut alpha_ty = 0;
            let mut depth_sz = 0;
            let mut depth_ty = 0;

            if ctxt.version >= &Version(Api::Gl, 4, 5) ||
               ctxt.extensions.gl_arb_direct_state_access
            {
                let id = texture.get_id();
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_RED_SIZE, &mut red_sz);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_RED_TYPE, &mut red_ty);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_GREEN_SIZE, &mut green_sz);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_GREEN_TYPE, &mut green_ty);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_BLUE_SIZE, &mut blue_sz);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_BLUE_TYPE, &mut blue_ty);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_ALPHA_SIZE, &mut alpha_sz);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_ALPHA_TYPE, &mut alpha_ty);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_DEPTH_SIZE, &mut depth_sz);
                ctxt.gl.GetTextureLevelParameteriv(id, 0, gl::TEXTURE_DEPTH_TYPE, &mut depth_ty);

            } else if ctxt.extensions.gl_ext_direct_state_access {
                let id = texture.get_id();
                let bind_point = texture.get_bind_point();

                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_RED_SIZE,
                                                      &mut red_sz);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_RED_TYPE,
                                                      &mut red_ty);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_GREEN_SIZE,
                                                      &mut green_sz);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_GREEN_TYPE,
                                                      &mut green_ty);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_BLUE_SIZE,
                                                      &mut blue_sz);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_BLUE_TYPE,
                                                      &mut blue_ty);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_ALPHA_SIZE,
                                                      &mut alpha_sz);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_ALPHA_TYPE,
                                                      &mut alpha_ty);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_DEPTH_SIZE,
                                                      &mut depth_sz);
                ctxt.gl.GetTextureLevelParameterivEXT(id, bind_point, 0, gl::TEXTURE_DEPTH_TYPE,
                                                      &mut depth_ty);

            } else {
                let bind_point = texture.bind_to_current(ctxt);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_RED_SIZE, &mut red_sz);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_RED_TYPE, &mut red_ty);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_GREEN_SIZE,
                                               &mut green_sz);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_GREEN_TYPE,
                                               &mut green_ty);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_BLUE_SIZE, &mut blue_sz);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_BLUE_TYPE, &mut blue_ty);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_ALPHA_SIZE,
                                               &mut alpha_sz);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_ALPHA_TYPE,
                                               &mut alpha_ty);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_DEPTH_SIZE,
                                               &mut depth_sz);
                ctxt.gl.GetTexLevelParameteriv(bind_point, 0, gl::TEXTURE_DEPTH_TYPE,
                                               &mut depth_ty);
            }

            (red_sz as gl::types::GLenum, red_ty as gl::types::GLenum,
             green_sz as gl::types::GLenum, green_ty as gl::types::GLenum,
             blue_sz as gl::types::GLenum, blue_ty as gl::types::GLenum,
             alpha_sz as gl::types::GLenum, alpha_ty as gl::types::GLenum,
             depth_sz as gl::types::GLenum, depth_ty as gl::types::GLenum)
        };

        Ok(match (red_sz, red_ty, green_sz, green_ty, blue_sz, blue_ty,
               alpha_sz, alpha_ty, depth_sz, depth_ty)
        {
            (_, gl::NONE, _, _, _, _, _, _, sz1, ty1) => InternalFormat::OneComponent {
                ty1: InternalFormatType::from_glenum(ty1),
                bits1: sz1 as usize,
            },
            (sz1, ty1, _, gl::NONE, _, _, _, _, _, _) => InternalFormat::OneComponent {
                ty1: InternalFormatType::from_glenum(ty1),
                bits1: sz1 as usize,
            },
            (sz1, ty1, sz2, ty2, _, gl::NONE, _, _, _, _) => InternalFormat::TwoComponents {
                ty1: InternalFormatType::from_glenum(ty1),
                bits1: sz1 as usize,
                ty2: InternalFormatType::from_glenum(ty2),
                bits2: sz2 as usize,
            },
            (sz1, ty1, sz2, ty2, sz3, ty3, _, gl::NONE, _, _) => InternalFormat::ThreeComponents {
                ty1: InternalFormatType::from_glenum(ty1),
                bits1: sz1 as usize,
                ty2: InternalFormatType::from_glenum(ty2),
                bits2: sz2 as usize,
                ty3: InternalFormatType::from_glenum(ty3),
                bits3: sz3 as usize,
            },
            (sz1, ty1, sz2, ty2, sz3, ty3, sz4, ty4, _, gl::NONE) => InternalFormat::FourComponents {
                ty1: InternalFormatType::from_glenum(ty1),
                bits1: sz1 as usize,
                ty2: InternalFormatType::from_glenum(ty2),
                bits2: sz2 as usize,
                ty3: InternalFormatType::from_glenum(ty3),
                bits3: sz3 as usize,
                ty4: InternalFormatType::from_glenum(ty4),
                bits4: sz4 as usize,
            },
            _ => unreachable!()
        })

    } else {
        // FIXME: GL2
        Err(GetFormatError::NotSupported)
    }
}
