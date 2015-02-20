use Handle;
use gl;

use std::default::Default;

/// Represents the current OpenGL state.
///
/// The current state is passed to each function and can be freely updated.
pub struct GLState {
    /// Whether GL_BLEND is enabled
    pub enabled_blend: bool,

    /// Whether GL_CULL_FACE is enabled
    pub enabled_cull_face: bool,

    /// Whether GL_DEBUG_OUTPUT is enabled. None means "unknown".
    pub enabled_debug_output: Option<bool>,

    /// Whether GL_DEBUG_OUTPUT_SYNCHRONOUS is enabled
    pub enabled_debug_output_synchronous: bool,

    /// Whether GL_DEPTH_TEST is enabled
    pub enabled_depth_test: bool,

    /// Whether GL_DITHER is enabled
    pub enabled_dither: bool,

    /// Whether GL_MULTISAMPLE is enabled
    pub enabled_multisample: bool,

    /// Whether GL_POLYGON_OFFSET_FILL is enabled
    pub enabled_polygon_offset_fill: bool,

    /// Whether GL_RASTERIZER_DISCARD is enabled
    pub enabled_rasterizer_discard: bool,

    /// Whether GL_SAMPLE_ALPHA_TO_COVERAGE is enabled
    pub enabled_sample_alpha_to_coverage: bool,

    /// Whether GL_SAMPLE_COVERAGE is enabled
    pub enabled_sample_coverage: bool,

    /// Whether GL_SCISSOR_TEST is enabled
    pub enabled_scissor_test: bool,

    /// Whether GL_STENCIL_TEST is enabled
    pub enabled_stencil_test: bool,

    // The latest value passed to `glUseProgram`.
    pub program: Handle,

    // The latest value passed to `glBindVertexArray`.
    pub vertex_array: gl::types::GLuint,

    // The latest value passed to `glClearColor`.
    pub clear_color: (gl::types::GLclampf, gl::types::GLclampf,
                      gl::types::GLclampf, gl::types::GLclampf),

    // The latest value passed to `glClearDepthf`.
    pub clear_depth: gl::types::GLclampf,

    // The latest value passed to `glClearStencil`.
    pub clear_stencil: gl::types::GLint,

    /// The latest buffer bound to `GL_ARRAY_BUFFER`.
    pub array_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_PIXEL_PACK_BUFFER`.
    pub pixel_pack_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_PIXEL_UNPACK_BUFFER`.
    pub pixel_unpack_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_UNIFORM_BUFFER`.
    pub uniform_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_READ_FRAMEBUFFER`.
    pub read_framebuffer: gl::types::GLuint,

    /// The latest buffer bound to `GL_DRAW_FRAMEBUFFER`.
    pub draw_framebuffer: gl::types::GLuint,

    /// The latest values passed to `glReadBuffer` with the default framebuffer.
    /// `None` means "unknown".
    pub default_framebuffer_read: Option<gl::types::GLenum>,

    /// The latest render buffer bound with `glBindRenderbuffer`.
    pub renderbuffer: gl::types::GLuint,

    /// The latest values passed to `glBlendEquation`.
    pub blend_equation: gl::types::GLenum,

    /// The latest values passed to `glBlendFunc`.
    pub blend_func: (gl::types::GLenum, gl::types::GLenum),

    /// The latest value passed to `glDepthFunc`.
    pub depth_func: gl::types::GLenum,

    /// The latest value passed to `glDepthMask`.
    pub depth_mask: bool,

    /// The latest values passed to `glDepthRange`.
    pub depth_range: (f32, f32),

    /// The latest values passed to `glViewport`. `None` means unknown.
    pub viewport: Option<(gl::types::GLint, gl::types::GLint,
                          gl::types::GLsizei, gl::types::GLsizei)>,

    /// The latest values passed to `glScissor`. `None` means unknown.
    pub scissor: Option<(gl::types::GLint, gl::types::GLint,
                         gl::types::GLsizei, gl::types::GLsizei)>,

    /// The latest value passed to `glLineWidth`.
    pub line_width: gl::types::GLfloat,

    /// The latest value passed to `glCullFace`.
    pub cull_face: gl::types::GLenum,

    /// The latest value passed to `glPolygonMode`.
    pub polygon_mode: gl::types::GLenum,

    /// The latest value passed to `glPixelStore` with `GL_UNPACK_ALIGNMENT`.
    pub pixel_store_unpack_alignment: gl::types::GLint,

    /// The latest value passed to `glPixelStore` with `GL_PACK_ALIGNMENT`.
    pub pixel_store_pack_alignment: gl::types::GLint,

    /// The latest value passed to `glPatchParameter` with `GL_PATCH_VERTICES`.
    pub patch_patch_vertices: gl::types::GLint,

    /// The latest value passed to `glActiveTexture`.
    pub active_texture: gl::types::GLenum,
}

impl Default for GLState {
    fn default() -> GLState {
        GLState {
            enabled_blend: false,
            enabled_cull_face: false,
            enabled_debug_output: None,
            enabled_debug_output_synchronous: false,
            enabled_depth_test: false,
            enabled_dither: false,
            enabled_multisample: true,
            enabled_polygon_offset_fill: false,
            enabled_rasterizer_discard: false,
            enabled_sample_alpha_to_coverage: false,
            enabled_sample_coverage: false,
            enabled_scissor_test: false,
            enabled_stencil_test: false,

            program: Handle::Id(0),
            vertex_array: 0,
            clear_color: (0.0, 0.0, 0.0, 0.0),
            clear_depth: 1.0,
            clear_stencil: 0,
            array_buffer_binding: 0,
            pixel_pack_buffer_binding: 0,
            pixel_unpack_buffer_binding: 0,
            uniform_buffer_binding: 0,
            read_framebuffer: 0,
            draw_framebuffer: 0,
            default_framebuffer_read: None,
            renderbuffer: 0,
            depth_func: gl::LESS,
            depth_mask: true,
            depth_range: (0.0, 1.0),
            blend_equation: gl::FUNC_ADD,
            blend_func: (gl::ONE, gl::ZERO),
            viewport: None,
            scissor: None,
            line_width: 1.0,
            cull_face: gl::BACK,
            polygon_mode: gl::FILL,
            pixel_store_unpack_alignment: 4,
            pixel_store_pack_alignment: 4,
            patch_patch_vertices: 3,
            active_texture: gl::TEXTURE0,
        }
    }
}
