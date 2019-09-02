use context::ExtensionsList;
use version::Version;
use version::Api;

use std::cmp;
use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::BuildHasherDefault;

use fnv::FnvHasher;

use gl;
use ToGlEnum;

use CapabilitiesSource;
use image_format::TextureFormat;

/// Describes the OpenGL context profile.
#[derive(Debug, Copy, Clone)]
pub enum Profile {
    /// The context uses only future-compatible functions and definitions.
    Core,
    /// The context includes all immediate mode functions and definitions.
    Compatibility
}

/// Represents the capabilities of the context.
///
/// Contrary to the state, these values never change.
#[derive(Debug)]
pub struct Capabilities {
    /// List of versions of GLSL that are supported by the compiler.
    ///
    /// An empty list means that the backend doesn't have a compiler.
    pub supported_glsl_versions: Vec<Version>,

    /// Returns a version or release number. Vendor-specific information may follow the version
    /// number.
    pub version: String,

    /// The company responsible for this GL implementation.
    pub vendor: String,

    /// The name of the renderer. This name is typically specific to a particular
    /// configuration of a hardware platform.
    pub renderer: String,

    /// The OpenGL context profile if available.
    ///
    /// The context profile is available from OpenGL 3.2 onwards. `None` if not supported.
    pub profile: Option<Profile>,

    /// The context is in debug mode, which may have additional error and performance issue
    /// reporting functionality.
    pub debug: bool,

    /// The context is in "forward-compatible" mode, which means that no deprecated functionality
    /// will be supported.
    pub forward_compatible: bool,

    /// True if out-of-bound access on the GPU side can't result in crashes.
    pub robustness: bool,

    /// True if it is possible for the OpenGL context to be lost.
    pub can_lose_context: bool,

    /// What happens when you change the current OpenGL context.
    pub release_behavior: ReleaseBehavior,

    /// Whether the context supports left and right buffers.
    pub stereo: bool,

    /// True if the default framebuffer is in sRGB.
    pub srgb: bool,

    /// Number of bits in the default framebuffer's depth buffer
    pub depth_bits: Option<u16>,

    /// Number of bits in the default framebuffer's stencil buffer
    pub stencil_bits: Option<u16>,

    /// Informations about formats when used to create textures.
    pub internal_formats_textures: HashMap<TextureFormat, FormatInfos, BuildHasherDefault<FnvHasher>>,

    /// Informations about formats when used to create renderbuffers.
    pub internal_formats_renderbuffers: HashMap<TextureFormat, FormatInfos, BuildHasherDefault<FnvHasher>>,

    /// Maximum number of textures that can be bound to a program.
    ///
    /// `glActiveTexture` must be between `GL_TEXTURE0` and `GL_TEXTURE0` + this value - 1.
    pub max_combined_texture_image_units: gl::types::GLint,

    /// Maximum value for `GL_TEXTURE_MAX_ANISOTROPY_EXT​`.
    ///
    /// `None` if the extension is not supported by the hardware.
    pub max_texture_max_anisotropy: Option<gl::types::GLfloat>,

    /// Maximum size of a buffer texture. `None` if this is not supported.
    pub max_texture_buffer_size: Option<gl::types::GLint>,

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

    /// Number of work groups for compute shaders.
    pub max_compute_work_group_count: (gl::types::GLint, gl::types::GLint, gl::types::GLint),

    /// Maximum number of color attachment bind points.
    pub max_color_attachments: gl::types::GLint,

    /// Maximum width of an empty framebuffer. `None` if not supported.
    pub max_framebuffer_width: Option<gl::types::GLint>,

    /// Maximum height of an empty framebuffer. `None` if not supported.
    pub max_framebuffer_height: Option<gl::types::GLint>,

    /// Maximum layers of an empty framebuffer. `None` if not supported.
    pub max_framebuffer_layers: Option<gl::types::GLint>,

    /// Maximum samples of an empty framebuffer. `None` if not supported.
    pub max_framebuffer_samples: Option<gl::types::GLint>,
}

/// Information about an internal format.
#[derive(Debug)]
pub struct FormatInfos {
    /// Possible values for multisampling. `None` if unknown.
    pub multisamples: Option<Vec<gl::types::GLint>>,
}

/// Defines what happens when you change the current context.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReleaseBehavior {
    /// Nothing is done when using another context.
    None,

    /// The commands queue of the current context is flushed.
    Flush,
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
    // GL_CONTEXT_FLAGS are only available from GL 3.0 onwards
    let (debug, forward_compatible) = if version >= &Version(Api::Gl, 3, 0) {
        let mut val = 0;
        gl.GetIntegerv(gl::CONTEXT_FLAGS, &mut val);
        let val = val as gl::types::GLenum;
        ((val & gl::CONTEXT_FLAG_DEBUG_BIT) != 0,
         (val & gl::CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT) != 0)
    } else {
        (false, false)
    };

    // getting the value of `GL_RENDERER`
    let renderer = {
        let s = gl.GetString(gl::RENDERER);
        assert!(!s.is_null());
        String::from_utf8(CStr::from_ptr(s as *const _).to_bytes().to_vec()).ok()
                                    .expect("glGetString(GL_RENDERER) returned a non-UTF8 string")
    };

    Capabilities {
        supported_glsl_versions: {
            get_supported_glsl(gl, version, extensions)
        },

        version: {
            let s = gl.GetString(gl::VERSION);
            assert!(!s.is_null());
            String::from_utf8(CStr::from_ptr(s as *const _).to_bytes().to_vec()).ok()
                                        .expect("glGetString(GL_VERSION) returned a non-UTF8 string")
        },

        vendor: {
            let s = gl.GetString(gl::VENDOR);
            assert!(!s.is_null());
            String::from_utf8(CStr::from_ptr(s as *const _).to_bytes().to_vec()).ok()
                                        .expect("glGetString(GL_VENDOR) returned a non-UTF8 string")
        },

        profile: {
            if version >= &Version(Api::Gl, 3, 2) {
                let mut val = 0;
                gl.GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut val);
                let val = val as gl::types::GLenum;
                if (val & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT) != 0 {
                    Some(Profile::Compatibility)
                } else if (val & gl::CONTEXT_CORE_PROFILE_BIT) != 0 {
                    Some(Profile::Core)
                } else {
                    None
                }
            } else {
                None
            }
        },

        debug: debug,

        forward_compatible: forward_compatible,

        robustness: if version >= &Version(Api::Gl, 4, 5) || version >= &Version(Api::GlEs, 3, 2) ||
                       (version >= &Version(Api::Gl, 3, 0) && extensions.gl_arb_robustness)
        {
            // TODO: there seems to be no way to query `GL_CONTEXT_FLAGS` before OpenGL 3.0, even
            //       if `GL_ARB_robustness` is there
            let mut val = 0;
            gl.GetIntegerv(gl::CONTEXT_FLAGS, &mut val);
            let val = val as gl::types::GLenum;
            (val & gl::CONTEXT_FLAG_ROBUST_ACCESS_BIT) != 0

        } else if extensions.gl_khr_robustness || extensions.gl_ext_robustness {
            let mut val = 0;
            gl.GetBooleanv(gl::CONTEXT_ROBUST_ACCESS, &mut val);
            val != 0

        } else {
            false
        },

        can_lose_context: if version >= &Version(Api::Gl, 4, 5) || extensions.gl_khr_robustness ||
                             extensions.gl_arb_robustness || extensions.gl_ext_robustness
        {
            let mut val = 0;
            gl.GetIntegerv(gl::RESET_NOTIFICATION_STRATEGY, &mut val);

            match val as gl::types::GLenum {
                gl::LOSE_CONTEXT_ON_RESET => true,
                gl::NO_RESET_NOTIFICATION => false,

                // WORK-AROUND: AMD drivers erroneously return this value, which doesn't even
                //              correspond to any GLenum in the specs. We work around this bug
                //              by interpreting it as `false`.
                0x31BE => false,

                // WORK-AROUND: Adreno 430/506 drivers return NO_ERROR.
                gl::NO_ERROR => false,

                _ => unreachable!()
            }

        } else {
            false
        },

        release_behavior: if extensions.gl_khr_context_flush_control {
            let mut val = 0;
            gl.GetIntegerv(gl::CONTEXT_RELEASE_BEHAVIOR, &mut val);

            match val as gl::types::GLenum {
                gl::NONE => ReleaseBehavior::None,
                gl::CONTEXT_RELEASE_BEHAVIOR_FLUSH => ReleaseBehavior::Flush,
                _ => unreachable!()
            }

        } else {
            ReleaseBehavior::Flush
        },

        stereo: {
            if version >= &Version(Api::Gl, 1, 0) {
                let mut val: gl::types::GLboolean = 0;
                gl.GetBooleanv(gl::STEREO, &mut val);
                val != 0
            } else {
                false
            }
        },

        srgb: {
            // `glGetFramebufferAttachmentParameteriv` incorrectly returns GL_INVALID_ENUM on some
            // drivers, so we prefer using `glGetIntegerv` if possible.
            if version >= &Version(Api::Gl, 3, 0) && !extensions.gl_ext_framebuffer_srgb {
                let mut value = 0;
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::FRONT_LEFT,
                                                       gl::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING,
                                                       &mut value);
                value as gl::types::GLenum == gl::SRGB

            } else if extensions.gl_ext_framebuffer_srgb {
                let mut value = 0;
                gl.GetBooleanv(gl::FRAMEBUFFER_SRGB_CAPABLE_EXT, &mut value);
                value != 0

            } else {
                false
            }
        },

        depth_bits: {
            let mut value = 0;

            // `glGetFramebufferAttachmentParameteriv` incorrectly returns GL_INVALID_ENUM on some
            // drivers, so we prefer using `glGetIntegerv` if possible.
            //
            // Also note that `gl_arb_es2_compatibility` may provide `GL_DEPTH_BITS` but os/x
            // doesn't even though it provides this extension. I'm not sure whether this is a bug
            // with OS/X or just the extension actually not providing it.
            if version >= &Version(Api::Gl, 3, 0) && !extensions.gl_arb_compatibility {
                let mut ty = 0;
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::DEPTH,
                                                       gl::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE,
                                                       &mut ty);

                if ty as gl::types::GLenum == gl::NONE {
                    value = 0;
                } else {
                    gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::DEPTH,
                                                           gl::FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE,
                                                           &mut value);
                }

            } else {
                gl.GetIntegerv(gl::DEPTH_BITS, &mut value);
            };

            match value {
                0 => None,
                v => Some(v as u16),
            }
        },

        stencil_bits: {
            let mut value = 0;

            // `glGetFramebufferAttachmentParameteriv` incorrectly returns GL_INVALID_ENUM on some
            // drivers, so we prefer using `glGetIntegerv` if possible.
            //
            // Also note that `gl_arb_es2_compatibility` may provide `GL_STENCIL_BITS` but os/x
            // doesn't even though it provides this extension. I'm not sure whether this is a bug
            // with OS/X or just the extension actually not providing it.
            if version >= &Version(Api::Gl, 3, 0) && !extensions.gl_arb_compatibility {
                let mut ty = 0;
                gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::STENCIL,
                                                       gl::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE,
                                                       &mut ty);

                if ty as gl::types::GLenum == gl::NONE {
                    value = 0;
                } else {
                    gl.GetFramebufferAttachmentParameteriv(gl::FRAMEBUFFER, gl::STENCIL,
                                                           gl::FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE,
                                                           &mut value);
                }

            } else {
                gl.GetIntegerv(gl::STENCIL_BITS, &mut value);
            };

            match value {
                0 => None,
                v => Some(v as u16),
            }
        },

        internal_formats_textures: get_internal_formats(gl, version, extensions, false),
        internal_formats_renderbuffers: get_internal_formats(gl, version, extensions, true),

        max_combined_texture_image_units: {
            let mut val = 2;
            gl.GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut val);

            // WORK-AROUND (issue #1181)
            // Some Radeon drivers crash if you use texture units 32 or more.
            if renderer.contains("Radeon") {
                val = cmp::min(val, 32);
            }

            val
        },

        max_texture_max_anisotropy: if !extensions.gl_ext_texture_filter_anisotropic {
            None

        } else {
            Some({
                let mut val = 0.0;
                gl.GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut val);
                val
            })
        },

        max_texture_buffer_size: {
            if version >= &Version(Api::Gl, 3, 0) || extensions.gl_arb_texture_buffer_object ||
               extensions.gl_ext_texture_buffer_object || extensions.gl_oes_texture_buffer ||
               extensions.gl_ext_texture_buffer
            {
                Some({
                    let mut val = 0;
                    gl.GetIntegerv(gl::MAX_TEXTURE_BUFFER_SIZE, &mut val);
                    val
                })

            } else {
                None
            }
        },

        max_viewport_dims: {
            let mut val: [gl::types::GLint; 2] = [ 0, 0 ];
            gl.GetIntegerv(gl::MAX_VIEWPORT_DIMS, val.as_mut_ptr());
            (val[0], val[1])
        },

        max_draw_buffers: {
            if version >= &Version(Api::Gl, 2, 0) ||
                version >= &Version(Api::GlEs, 3, 0) ||
                extensions.gl_ati_draw_buffers || extensions.gl_arb_draw_buffers
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
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_PATCH_VERTICES, &mut val);
                val
            })

        } else {
            None
        },

        max_indexed_atomic_counter_buffer: if version >= &Version(Api::Gl, 4, 2) {      // TODO: ARB_shader_atomic_counters   // TODO: GLES
            let mut val = 0;
            gl.GetIntegerv(gl::MAX_ATOMIC_COUNTER_BUFFER_BINDINGS, &mut val);
            val
        } else {
            0
        },

        max_indexed_shader_storage_buffer: {
            if version >= &Version(Api::Gl, 4, 3) || extensions.gl_arb_shader_storage_buffer_object {      // TODO: GLES
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_SHADER_STORAGE_BUFFER_BINDINGS, &mut val);
                val
            } else {
                0
            }
        },

        max_indexed_transform_feedback_buffer: {
            if version >= &Version(Api::Gl, 4, 0) || extensions.gl_arb_transform_feedback3 {      // TODO: GLES
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_TRANSFORM_FEEDBACK_BUFFERS, &mut val);
                val
            } else if version >= &Version(Api::Gl, 3, 0) || extensions.gl_ext_transform_feedback {
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS_EXT, &mut val);
                val
            } else {
                0
            }
        },

        max_indexed_uniform_buffer: {
            if version >= &Version(Api::Gl, 3, 1) || extensions.gl_arb_uniform_buffer_object {      // TODO: GLES
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_UNIFORM_BUFFER_BINDINGS, &mut val);
                val
            } else {
                0
            }
        },

        max_compute_work_group_count: if version >= &Version(Api::Gl, 4, 3) ||
                                         version >= &Version(Api::GlEs, 3, 1) ||
                                         extensions.gl_arb_compute_shader
        {
            let mut val1 = 0;
            let mut val2 = 0;
            let mut val3 = 0;
            gl.GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 0, &mut val1);
            gl.GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 1, &mut val2);
            gl.GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 2, &mut val3);
            (val1, val2, val3)

        } else {
            (0, 0, 0)
        },

        max_color_attachments: {
            if version >= &Version(Api::Gl, 3, 0) || version >= &Version(Api::GlEs, 3, 0) ||
               extensions.gl_arb_framebuffer_object || extensions.gl_ext_framebuffer_object ||
               extensions.gl_nv_fbo_color_attachments
            {
                let mut val = 4;
                gl.GetIntegerv(gl::MAX_COLOR_ATTACHMENTS, &mut val);
                val
            } else if version >= &Version(Api::GlEs, 2, 0) {
                1
            } else {
                // glium doesn't allow creating contexts that don't support FBOs
                unreachable!()
            }
        },

        max_framebuffer_width: {
            if version >= &Version(Api::Gl, 4, 3) || version >= &Version(Api::GlEs, 3, 1) ||
               extensions.gl_arb_framebuffer_no_attachments
            {
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_FRAMEBUFFER_WIDTH, &mut val);
                Some(val)

            } else {
                None
            }
        },

        max_framebuffer_height: {
            if version >= &Version(Api::Gl, 4, 3) || version >= &Version(Api::GlEs, 3, 1) ||
               extensions.gl_arb_framebuffer_no_attachments
            {
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_FRAMEBUFFER_HEIGHT, &mut val);
                Some(val)

            } else {
                None
            }
        },

        max_framebuffer_layers: {
            if version >= &Version(Api::Gl, 4, 3) || version >= &Version(Api::GlEs, 3, 2) ||
               extensions.gl_arb_framebuffer_no_attachments
            {
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_FRAMEBUFFER_LAYERS, &mut val);
                Some(val)

            } else {
                None
            }
        },

        max_framebuffer_samples: {
            if version >= &Version(Api::Gl, 4, 3) || version >= &Version(Api::GlEs, 3, 1) ||
               extensions.gl_arb_framebuffer_no_attachments
            {
                let mut val = 0;
                gl.GetIntegerv(gl::MAX_FRAMEBUFFER_SAMPLES, &mut val);
                Some(val)

            } else {
                None
            }
        },

        renderer: renderer,
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
        let mut val = 0;
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

    if version >= &Version(Api::GlEs, 3, 2) || extensions.gl_arb_es3_2_compatibility {
        result.push(Version(Api::GlEs, 3, 2));
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

/// Returns all informations about all supported internal formats.
pub fn get_internal_formats(gl: &gl::Gl, version: &Version, extensions: &ExtensionsList,
                            renderbuffer: bool) -> HashMap<TextureFormat, FormatInfos, BuildHasherDefault<FnvHasher>>
{
    // We create a dummy object to implement the `CapabilitiesSource` trait.
    let dummy = {
        struct DummyCaps<'a>(&'a Version, &'a ExtensionsList);
        impl<'a> CapabilitiesSource for DummyCaps<'a> {
            fn get_version(&self) -> &Version { self.0 }
            fn get_extensions(&self) -> &ExtensionsList { self.1 }
            fn get_capabilities(&self) -> &Capabilities { unreachable!() }
        }
        DummyCaps(version, extensions)
    };

    TextureFormat::get_formats_list().into_iter().filter_map(|format| {
        if renderbuffer {
            if !format.is_supported_for_renderbuffers(&dummy) {
                return None;
            }
        } else {
            if !format.is_supported_for_textures(&dummy) {
                return None;
            }
        }

        let infos = get_internal_format(gl, version, extensions, format, renderbuffer);
        Some((format, infos))
    }).collect()
}

/// Returns informations about a precise internal format.
pub fn get_internal_format(gl: &gl::Gl, version: &Version, extensions: &ExtensionsList,
                           format: TextureFormat, renderbuffer: bool) -> FormatInfos
{
    // We create a dummy object to implement the `CapabilitiesSource` trait.
    let dummy = {
        struct DummyCaps<'a>(&'a Version, &'a ExtensionsList);
        impl<'a> CapabilitiesSource for DummyCaps<'a> {
            fn get_version(&self) -> &Version { self.0 }
            fn get_extensions(&self) -> &ExtensionsList { self.1 }
            fn get_capabilities(&self) -> &Capabilities { unreachable!() }
        }
        DummyCaps(version, extensions)
    };

    unsafe {
        let target = if renderbuffer { gl::RENDERBUFFER } else { gl::TEXTURE_2D_MULTISAMPLE };

        let samples = if format.is_renderable(&dummy) &&
                         ((version >= &Version(Api::GlEs, 3, 0) && renderbuffer) ||
                          version >= &Version(Api::Gl, 4, 2) ||
                          extensions.gl_arb_internalformat_query)
        {
            let mut num = 0;
            gl.GetInternalformativ(target, format.to_glenum(), gl::NUM_SAMPLE_COUNTS, 1, &mut num);

            if num >= 1 {
                let mut formats = Vec::with_capacity(num as usize);
                gl.GetInternalformativ(target, format.to_glenum(), gl::SAMPLES, num,
                                       formats.as_mut_ptr());
                formats.set_len(num as usize);
                Some(formats)

            } else {
                Some(Vec::new())
            }

        } else {
            None
        };

        FormatInfos {
            multisamples: samples,
        }
    }
}
