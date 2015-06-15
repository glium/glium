use Handle;
use gl;

/// Represents the current OpenGL state.
///
/// The current state is passed to each function and can be freely updated.
pub struct GlState {
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

    /// Whether GL_FRAMEBUFFER_SRGB is enabled
    pub enabled_framebuffer_srgb: bool,

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

    /// Whether GL_LINE_SMOOTH is enabled
    pub enabled_line_smooth: bool,

    /// Whether GL_POLYGON_SMOOTH is enabled
    pub enabled_polygon_smooth: bool,

    /// The latest value passed to `glUseProgram`.
    pub program: Handle,

    /// The latest value passed to `glBindVertexArray`.
    pub vertex_array: gl::types::GLuint,

    /// The latest value passed to `glClearColor`.
    pub clear_color: (gl::types::GLclampf, gl::types::GLclampf,
                      gl::types::GLclampf, gl::types::GLclampf),

    /// The latest value passed to `glClearDepthf`.
    pub clear_depth: gl::types::GLclampf,

    /// The latest value passed to `glClearStencil`.
    pub clear_stencil: gl::types::GLint,

    /// The latest values passed to ``glColorMask`.
    pub color_mask: (gl::types::GLboolean, gl::types::GLboolean,
                     gl::types::GLboolean, gl::types::GLboolean),

    /// The latest buffer bound to `GL_ARRAY_BUFFER`.
    pub array_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_PIXEL_PACK_BUFFER`.
    pub pixel_pack_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_PIXEL_UNPACK_BUFFER`.
    pub pixel_unpack_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_UNIFORM_BUFFER`.
    pub uniform_buffer_binding: gl::types::GLuint,

    /// List of buffers bound to the indexed `GL_UNIFORM_BUFFER`.
    pub indexed_uniform_buffer_bindings: Vec<IndexedBufferState>,

    /// The latest buffer bound to `GL_COPY_READ_BUFFER`.
    pub copy_read_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_COPY_WRITE_BUFFER`.
    pub copy_write_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_DISPATCH_INDIRECT_BUFFER`.
    pub dispatch_indirect_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_DRAW_INDIRECT_BUFFER`.
    pub draw_indirect_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_QUERY_BUFFER`.
    pub query_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_TEXTURE_BUFFER`.
    pub texture_buffer_binding: gl::types::GLuint,

    /// The latest buffer bound to `GL_ATOMIC_COUNTER_BUFFER`.
    pub atomic_counter_buffer_binding: gl::types::GLuint,

    /// List of buffers bound to the indexed `GL_ATOMIC_COUNTER_BUFFER`.
    pub indexed_atomic_counter_buffer_bindings: Vec<IndexedBufferState>,

    /// The latest buffer bound to `GL_SHADER_STORAGE_BUFFER`.
    pub shader_storage_buffer_binding: gl::types::GLuint,

    /// List of buffers bound to the indexed `GL_SHADER_STORAGE_BUFFER`.
    pub indexed_shader_storage_buffer_bindings: Vec<IndexedBufferState>,

    /// List of buffers bound to the indexed `GL_TRANSFORM_FEEDBACK_BUFFER`.
    pub indexed_transform_feedback_buffer_bindings: Vec<IndexedBufferState>,

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

    /// The latest values passed to `glStencilFuncSeparate` with face `GL_FRONT`.
    pub stencil_func_front: (gl::types::GLenum, gl::types::GLint, gl::types::GLuint),

    /// The latest values passed to `glStencilFuncSeparate` with face `GL_BACK`.
    pub stencil_func_back: (gl::types::GLenum, gl::types::GLint, gl::types::GLuint),

    /// The latest value passed to `glStencilMaskSeparate` with face `GL_FRONT`.
    pub stencil_mask_front: gl::types::GLuint,

    /// The latest value passed to `glStencilMaskSeparate` with face `GL_BACK`.
    pub stencil_mask_back: gl::types::GLuint,

    /// The latest values passed to `glStencilOpSeparate` with face `GL_FRONT`.
    pub stencil_op_front: (gl::types::GLenum, gl::types::GLenum, gl::types::GLenum),

    /// The latest values passed to `glStencilOpSeparate` with face `GL_BACK`.
    pub stencil_op_back: (gl::types::GLenum, gl::types::GLenum, gl::types::GLenum),

    /// The latest values passed to `glViewport`. `None` means unknown.
    pub viewport: Option<(gl::types::GLint, gl::types::GLint,
                          gl::types::GLsizei, gl::types::GLsizei)>,

    /// The latest values passed to `glScissor`. `None` means unknown.
    pub scissor: Option<(gl::types::GLint, gl::types::GLint,
                         gl::types::GLsizei, gl::types::GLsizei)>,

    /// The latest value passed to `glLineWidth`.
    pub line_width: gl::types::GLfloat,

    /// The latest value passed to `glPointSize`.
    pub point_size: gl::types::GLfloat,

    /// The latest value passed to `glCullFace`.
    pub cull_face: gl::types::GLenum,

    /// The latest value passed to `glPolygonMode`.
    pub polygon_mode: gl::types::GLenum,

    /// The latest value passed to `glHint` for smoothing.
    pub smooth: (gl::types::GLenum, gl::types::GLenum),

    /// The latest value passed to `glPixelStore` with `GL_UNPACK_ALIGNMENT`.
    pub pixel_store_unpack_alignment: gl::types::GLint,

    /// The latest value passed to `glPixelStore` with `GL_PACK_ALIGNMENT`.
    pub pixel_store_pack_alignment: gl::types::GLint,

    /// The latest value passed to `glPatchParameter` with `GL_PATCH_VERTICES`.
    pub patch_patch_vertices: gl::types::GLint,

    /// The id of the active texture unit.
    /// IMPORTANT: this is a raw number (0, 1, 2, ...), not an
    ///            enumeration (GL_TEXTURE0, GL_TEXTURE1, ...).
    pub active_texture: gl::types::GLenum,

    /// List of texture units.
    pub texture_units: Vec<TextureUnitState>,

    /// Current query being used for GL_SAMPLES_PASSED​.
    pub samples_passed_query: gl::types::GLuint,

    /// Current query being used for GL_ANY_SAMPLES_PASSED​.
    pub any_samples_passed_query: gl::types::GLuint,

    /// Current query being used for GL_ANY_SAMPLES_PASSED​_CONSERVATIVE.
    pub any_samples_passed_conservative_query: gl::types::GLuint,

    /// Current query being used for GL_PRIMITIVES_GENERATED​.
    pub primitives_generated_query: gl::types::GLuint,

    /// Current query being used for GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN​.
    pub transform_feedback_primitives_written_query: gl::types::GLuint,

    /// Current query being used for GL_TIME_ELAPSED​.
    pub time_elapsed_query: gl::types::GLuint,

    /// Latest value passed to `glBeginConditionalRender​`.
    pub conditional_render: Option<(gl::types::GLuint, gl::types::GLenum)>,

    /// If `glBeginTransformFeedback​` has been called, the current primitive types. Otherwise None.
    // TODO: move this inside transform feedback objects
    pub transform_feedback_enabled: Option<gl::types::GLenum>,

    /// True if `glPauseTransformFeedback` has been called.
    // TODO: move this inside transform feedback objects
    pub transform_feedback_paused: bool,

    /// Current draw call ID.
    /// We maintain a counter that is incremented at each draw call.
    pub next_draw_call_id: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_VERTEX_ATTRIB_ARRAY_BARRIER_BIT`.
    pub latest_memory_barrier_vertex_attrib_array: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_ELEMENT_ARRAY_BARRIER_BIT`.
    pub latest_memory_barrier_element_array: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_UNIFORM_BARRIER_BIT`.
    pub latest_memory_barrier_uniform: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_TEXTURE_FETCH_BARRIER_BIT`.
    pub latest_memory_barrier_texture_fetch: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_SHADER_IMAGE_ACCESS_BARRIER_BIT`.
    pub latest_memory_barrier_shader_image_access: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_COMMAND_BARRIER_BIT`.
    pub latest_memory_barrier_command: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_PIXEL_BUFFER_BARRIER_BIT`.
    pub latest_memory_barrier_pixel_buffer: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_TEXTURE_UPDATE_BARRIER_BIT`.
    pub latest_memory_barrier_texture_update: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_BUFFER_UPDATE_BARRIER_BIT`.
    pub latest_memory_barrier_buffer_update: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_FRAMEBUFFER_BARRIER_BIT`.
    pub latest_memory_barrier_framebuffer: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_TRANSFORM_FEEDBACK_BARRIER_BIT`.
    pub latest_memory_barrier_transform_feedback: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_ATOMIC_COUNTER_BARRIER_BIT`.
    pub latest_memory_barrier_atomic_counter: u64,

    /// The draw call ID of the latest call to `glMemoryBarrier` with
    /// `GL_SHADER_STORAGE_BARRIER_BIT`.
    pub latest_memory_barrier_shader_storage: u64,
}

/// State of a texture unit (the one designated by `glActiveTexture`).
#[derive(Copy, Clone, Debug)]
pub struct TextureUnitState {
    /// Id of the texture.
    pub texture: gl::types::GLuint,

    /// Id of the sampler.
    pub sampler: gl::types::GLuint,
}

/// State of an indexed buffer target (`glBindBufferRange`/`glBindBufferBase`).
#[derive(Copy, Clone, Debug)]
pub struct IndexedBufferState {
    /// Id of the buffer.
    pub buffer: gl::types::GLuint,

    /// Starting offset in bytes.
    pub offset: gl::types::GLintptr,

    /// Size in bytes.
    pub size: gl::types::GLsizeiptr,
}

/// Builds the `GlState` corresponding to a newly-created OpenGL context.
impl Default for GlState {
    fn default() -> GlState {
        GlState {
            enabled_blend: false,
            enabled_cull_face: false,
            enabled_debug_output: None,
            enabled_debug_output_synchronous: false,
            enabled_depth_test: false,
            enabled_dither: false,
            enabled_framebuffer_srgb: false,
            enabled_multisample: true,
            enabled_polygon_offset_fill: false,
            enabled_rasterizer_discard: false,
            enabled_sample_alpha_to_coverage: false,
            enabled_sample_coverage: false,
            enabled_scissor_test: false,
            enabled_stencil_test: false,
            enabled_line_smooth: false,
            enabled_polygon_smooth: false,

            program: Handle::Id(0),
            vertex_array: 0,
            clear_color: (0.0, 0.0, 0.0, 0.0),
            clear_depth: 1.0,
            clear_stencil: 0,
            color_mask: (1, 1, 1, 1),
            array_buffer_binding: 0,
            pixel_pack_buffer_binding: 0,
            pixel_unpack_buffer_binding: 0,
            uniform_buffer_binding: 0,
            indexed_uniform_buffer_bindings: vec![Default::default()],
            copy_read_buffer_binding: 0,
            copy_write_buffer_binding: 0,
            dispatch_indirect_buffer_binding: 0,
            draw_indirect_buffer_binding: 0,
            query_buffer_binding: 0,
            texture_buffer_binding: 0,
            atomic_counter_buffer_binding: 0,
            indexed_atomic_counter_buffer_bindings: vec![Default::default()],
            shader_storage_buffer_binding: 0,
            indexed_shader_storage_buffer_bindings: vec![Default::default()],
            indexed_transform_feedback_buffer_bindings: vec![Default::default()],
            read_framebuffer: 0,
            draw_framebuffer: 0,
            default_framebuffer_read: None,
            renderbuffer: 0,
            depth_func: gl::LESS,
            depth_mask: true,
            depth_range: (0.0, 1.0),
            stencil_func_front: (gl::ALWAYS, 0, 0xffffffff),
            stencil_func_back: (gl::ALWAYS, 0, 0xffffffff),
            stencil_mask_front: 0xffffffff,
            stencil_mask_back: 0xffffffff,
            stencil_op_front: (gl::KEEP, gl::KEEP, gl::KEEP),
            stencil_op_back: (gl::KEEP, gl::KEEP, gl::KEEP),
            blend_equation: gl::FUNC_ADD,
            blend_func: (gl::ONE, gl::ZERO),
            viewport: None,
            scissor: None,
            line_width: 1.0,
            point_size: 1.0,
            cull_face: gl::BACK,
            polygon_mode: gl::FILL,
            smooth: (gl::DONT_CARE, gl::DONT_CARE),
            pixel_store_unpack_alignment: 4,
            pixel_store_pack_alignment: 4,
            patch_patch_vertices: 3,
            active_texture: 0,
            texture_units: vec![Default::default()],
            samples_passed_query: 0,
            any_samples_passed_query: 0,
            any_samples_passed_conservative_query: 0,
            primitives_generated_query: 0,
            transform_feedback_primitives_written_query: 0,
            time_elapsed_query: 0,
            conditional_render: None,
            transform_feedback_enabled: None,
            transform_feedback_paused: false,

            next_draw_call_id: 1,
            latest_memory_barrier_vertex_attrib_array: 1,
            latest_memory_barrier_element_array: 1,
            latest_memory_barrier_uniform: 1,
            latest_memory_barrier_texture_fetch: 1,
            latest_memory_barrier_shader_image_access: 1,
            latest_memory_barrier_command: 1,
            latest_memory_barrier_pixel_buffer: 1,
            latest_memory_barrier_texture_update: 1,
            latest_memory_barrier_buffer_update: 1,
            latest_memory_barrier_framebuffer: 1,
            latest_memory_barrier_transform_feedback: 1,
            latest_memory_barrier_atomic_counter: 1,
            latest_memory_barrier_shader_storage: 1,
        }
    }
}

impl Default for TextureUnitState {
    fn default() -> TextureUnitState {
        TextureUnitState {
            texture: 0,
            sampler: 0,
        }
    }
}

impl Default for IndexedBufferState {
    fn default() -> IndexedBufferState {
        IndexedBufferState {
            buffer: 0,
            offset: 0,
            size: 0,
        }
    }
}
