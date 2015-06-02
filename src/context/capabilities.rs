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

    /// True if the default framebuffer is in sRGB.
    pub srgb: bool,

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

    /// Number of available buffer bind points for `GL_ATOMIC_COUNTER_BUFFER`.
    pub max_indexed_atomic_counter_buffer: gl::types::GLint,

    /// Number of available buffer bind points for `GL_SHADER_STORAGE_BUFFER`.
    pub max_indexed_shader_storage_buffer: gl::types::GLint,

    /// Number of available buffer bind points for `GL_TRANSFORM_FEEDBACK_BUFFER`.
    pub max_indexed_transform_feedback_buffer: gl::types::GLint,

    /// Number of available buffer bind points for `GL_UNIFORM_BUFFER`.
    pub max_indexed_uniform_buffer: gl::types::GLint,
}

/// Loads the capabilities.
///
/// *Safety*: the OpenGL context corresponding to `gl` must be current in the thread.
///
/// ## Panic
///
/// Can panic if the version number or extensions list don't match the backend, leading to
/// unloaded functions being called.
///
pub unsafe fn get_capabilities(gl: &gl::Gl, version: &Version, extensions: &ExtensionsList)
                               -> Capabilities
{
    use std::mem;

    Capabilities {
        supported_glsl_versions: {
            get_supported_glsl(gl, version, extensions)
        },

        stereo: {
            if version >= &Version(Api::Gl, 1, 0) {
                let mut val: gl::types::GLboolean = mem::uninitialized();
                gl.GetBooleanv(gl::STEREO, &mut val);
                val != 0
            } else {
                false
            }
        },

        srgb: {
            if version >= &Version(Api::Gl, 3, 0) {
                let mut value = mem::uninitialized();
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::BACK_LEFT,
                                                       gl::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING,
                                                       &mut value);
                value as gl::types::GLenum == gl::SRGB

            } else if extensions.gl_ext_framebuffer_srgb {
                let mut value = mem::uninitialized();
                gl.GetBooleanv(gl::FRAMEBUFFER_SRGB_CAPABLE_EXT, &mut value);
                value != 0

            } else {
                false
            }
        },

        depth_bits: {
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

        stencil_bits: {
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

        max_combined_texture_image_units: {
            let mut val = 2;
            gl.GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut val);
            val
        },

        max_texture_max_anisotropy: if !extensions.gl_ext_texture_filter_anisotropic {
            None

        } else {
            Some({
                let mut val = mem::uninitialized();
                gl.GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut val);
                val
            })
        },

        max_viewport_dims: {
            let mut val: [gl::types::GLint; 2] = [ 0, 0 ];
            gl.GetIntegerv(gl::MAX_VIEWPORT_DIMS, val.as_mut_ptr());
            (val[0], val[1])
        },

        max_draw_buffers: {
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
            Some({
                let mut val = mem::uninitialized();
                gl.GetIntegerv(gl::MAX_PATCH_VERTICES, &mut val);
                val
            })

        } else {
            None
        },

        max_indexed_atomic_counter_buffer: if version >= &Version(Api::Gl, 4, 2) {      // TODO: ARB_shader_atomic_counters   // TODO: GLES
            let mut val = mem::uninitialized();
            gl.GetIntegerv(gl::MAX_ATOMIC_COUNTER_BUFFER_BINDINGS, &mut val);
            val
        } else {
            0
        },

        max_indexed_shader_storage_buffer: if version >= &Version(Api::Gl, 4, 3) {      // TODO: ARB_shader_storage_buffer_object   // TODO: GLES
            let mut val = mem::uninitialized();
            gl.GetIntegerv(gl::MAX_SHADER_STORAGE_BUFFER_BINDINGS, &mut val);
            val
        } else {
            0
        },

        max_indexed_transform_feedback_buffer: {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_transform_feedback3 {      // TODO: make sure that GL 3.0 supports it   // TODO: GLES
                let mut val = mem::uninitialized();
                gl.GetIntegerv(gl::MAX_TRANSFORM_FEEDBACK_BUFFERS, &mut val);
                val
            } else if extensions.gl_ext_transform_feedback {
                let mut val = mem::uninitialized();
                gl.GetIntegerv(gl::MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS_EXT, &mut val);
                val
            } else {
                0
            }
        },

        max_indexed_uniform_buffer: if version >= &Version(Api::Gl, 3, 0) {      // TODO: ARB_shader_storage_buffer_object   // TODO: GLES
            let mut val = mem::uninitialized();
            gl.GetIntegerv(gl::MAX_UNIFORM_BUFFER_BINDINGS, &mut val);
            val
        } else {
            0
        },
    }
}

/// Gets the list of GLSL versions supported by the backend.
///
/// *Safety*: the OpenGL context corresponding to `gl` must be current in the thread.
///
/// ## Panic
///
/// Can panic if the version number or extensions list don't match the backend, leading to
/// unloaded functions being called.
///
pub unsafe fn get_supported_glsl(gl: &gl::Gl, version: &Version, extensions: &ExtensionsList)
                                 -> Vec<Version>
{
    // checking if the implementation has a shader compiler
    // a compiler is optional in OpenGL ES
    if version.0 == Api::GlEs {
        let mut val = mem::uninitialized();
        gl.GetBooleanv(gl::SHADER_COMPILER, &mut val);
        if val == 0 {
            return vec![];
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
