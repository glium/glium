use context::ExtensionsList;
use version::Version;
use version::Api;
use std::mem;
use gl;

/// Represents the capabilities of the context.
pub struct Capabilities {
    /// List of versions of GLSL that are supported by the compiler.
    ///
    /// An empty list means that the backend doesn't have a compiler.
    pub supported_glsl_versions: Vec<Version>,

    /// Whether the context supports left and right buffers.
    pub stereo: bool,

    /// Number of bits in the default framebuffer's depth buffer
    pub depth_bits: Option<u16>,

    /// Number of bits in the default framebuffer's stencil buffer
    pub stencil_bits: Option<u16>,

    /// Maximum number of textures that can be bound to a program.
    ///
    /// `glActiveTexture` must be between `GL_TEXTURE0` and `GL_TEXTURE0` + this value - 1.
    pub max_combined_texture_image_units: gl::types::GLint,

    /// Maximum value for `GL_TEXTURE_MAX_ANISOTROPY_EXT​`.
    ///
    /// `None` if the extension is not supported by the hardware.
    pub max_texture_max_anisotropy: Option<gl::types::GLfloat>,

    /// Maximum width and height of `glViewport`.
    pub max_viewport_dims: (gl::types::GLint, gl::types::GLint),

    /// Maximum number of elements that can be passed with `glDrawBuffers`.
    pub max_draw_buffers: gl::types::GLint,

    /// Maximum number of vertices per patch. `None` if tessellation is not supported.
    pub max_patch_vertices: Option<gl::types::GLint>,
}

/// Loads the capabilities.
pub fn get_capabilities(gl: &gl::Gl, version: &Version, extensions: &ExtensionsList)
                        -> Capabilities
{
    use std::mem;

    Capabilities {
        supported_glsl_versions: {
            get_supported_glsl(gl, version, extensions)
        },

        stereo: unsafe {
            if version >= &Version(Api::Gl, 1, 0) {
                let mut val: gl::types::GLboolean = mem::uninitialized();
                gl.GetBooleanv(gl::STEREO, &mut val);
                val != 0
            } else {
                false
            }
        },

        depth_bits: unsafe {
            let mut value = mem::uninitialized();

            if version >= &Version(Api::Gl, 3, 0) {
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::DEPTH,
                                                       gl::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE,
                                                       &mut value);
            } else {
                gl.GetIntegerv(gl::DEPTH_BITS, &mut value);
            };

            match value {
                0 => None,
                v => Some(v as u16),
            }
        },

        stencil_bits: unsafe {
            let mut value = mem::uninitialized();

            if version >= &Version(Api::Gl, 3, 0) {
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::STENCIL,
                                                       gl::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE,
                                                       &mut value);
            } else {
                gl.GetIntegerv(gl::STENCIL_BITS, &mut value);
            };

            match value {
                0 => None,
                v => Some(v as u16),
            }
        },

        max_combined_texture_image_units: unsafe {
            let mut val = 2;
            gl.GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut val);
            val
        },

        max_texture_max_anisotropy: if !extensions.gl_ext_texture_filter_anisotropic {
            None

        } else {
            Some(unsafe {
                let mut val = mem::uninitialized();
                gl.GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut val);
                val
            })
        },

        max_viewport_dims: unsafe {
            let mut val: [gl::types::GLint; 2] = [ 0, 0 ];
            gl.GetIntegerv(gl::MAX_VIEWPORT_DIMS, val.as_mut_ptr());
            (val[0], val[1])
        },

        max_draw_buffers: unsafe {
            if version >= &Version(Api::Gl, 2, 0) ||
                version >= &Version(Api::GlEs, 3, 0)
            {
                let mut val = 1;
                gl.GetIntegerv(gl::MAX_DRAW_BUFFERS, &mut val);
                val
            } else {
                1
            }
        },

        max_patch_vertices: if version >= &Version(Api::Gl, 4, 0) ||
            extensions.gl_arb_tessellation_shader
        {
            Some(unsafe {
                let mut val = mem::uninitialized();
                gl.GetIntegerv(gl::MAX_PATCH_VERTICES, &mut val);
                val
            })

        } else {
            None
        },
    }
}

/// Gets the list of GLSL versions supported by the backend.
pub fn get_supported_glsl(gl: &gl::Gl, version: &Version, extensions: &ExtensionsList)
                          -> Vec<Version>
{
    // checking if the implementation has a shader compiler
    // a compiler is optional in OpenGL ES
    if version.0 == Api::GlEs {
        unsafe {
            let mut val = mem::uninitialized();
            gl.GetBooleanv(gl::SHADER_COMPILER, &mut val);
            if val == 0 {
                return vec![];
            }
        }
    }

    // some recent versions have an API to determine the list of supported versions
    if version >= &Version(Api::Gl, 4, 3) {
        // FIXME: implement this and return the result directly
    }

    let mut result = Vec::with_capacity(8);

    if version >= &Version(Api::GlEs, 2, 0) || version >= &Version(Api::Gl, 4, 1) ||
       extensions.gl_arb_es2_compatibility
    {
        result.push(Version(Api::GlEs, 1, 0));
    }

    if version >= &Version(Api::GlEs, 3, 0) || version >= &Version(Api::Gl, 4, 3) ||
       extensions.gl_arb_es3_compatibility
    {
        result.push(Version(Api::GlEs, 3, 0));
    }

    if version >= &Version(Api::GlEs, 3, 1) || version >= &Version(Api::Gl, 4, 5) ||
       extensions.gl_arb_es3_1_compatibility
    {
        result.push(Version(Api::GlEs, 3, 1));
    }

    if version >= &Version(Api::Gl, 2, 0) && version <= &Version(Api::Gl, 3, 0) ||
       extensions.gl_arb_compatibility
    {
        result.push(Version(Api::Gl, 1, 1));
    }

    if version >= &Version(Api::Gl, 2, 1) && version <= &Version(Api::Gl, 3, 0) ||
       extensions.gl_arb_compatibility
    {
        result.push(Version(Api::Gl, 1, 2));
    }

    if version == &Version(Api::Gl, 3, 0) || extensions.gl_arb_compatibility {
        result.push(Version(Api::Gl, 1, 3));
    }

    if version >= &Version(Api::Gl, 3, 1) {
        result.push(Version(Api::Gl, 1, 4));
    }

    if version >= &Version(Api::Gl, 3, 2) {
        result.push(Version(Api::Gl, 1, 5));
    }

    for &(major, minor) in &[(3, 3), (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5)] {
        if version >= &Version(Api::Gl, major, minor) {
            result.push(Version(Api::Gl, major, minor));
        }
    }

    result
}
