use std::ffi;
use gl;

/// Contains data about the list of extensions
#[derive(Debug, Clone, Copy)]
pub struct ExtensionsList {
    /// GL_EXT_direct_state_access
    pub gl_ext_direct_state_access: bool,
    /// GL_ARB_direct_state_access
    pub gl_arb_direct_state_access: bool,
    /// GL_EXT_framebuffer_object
    pub gl_ext_framebuffer_object: bool,
    /// GL_EXT_geometry_shader4
    pub gl_ext_geometry_shader4: bool,
    /// GL_EXT_framebuffer_blit
    pub gl_ext_framebuffer_blit: bool,
    /// GL_KHR_debug
    pub gl_khr_debug: bool,
    /// GL_NVX_gpu_memory_info
    pub gl_nvx_gpu_memory_info: bool,
    /// GL_ATI_meminfo
    pub gl_ati_meminfo: bool,
    /// GL_ARB_vertex_array_object
    pub gl_arb_vertex_array_object: bool,
    /// GL_ARB_sampler_objects
    pub gl_arb_sampler_objects: bool,
    /// GL_EXT_texture_filter_anisotropic
    pub gl_ext_texture_filter_anisotropic: bool,
    /// GL_ARB_texture_storage
    pub gl_arb_texture_storage: bool,
    /// GL_ARB_buffer_storage
    pub gl_arb_buffer_storage: bool,
    /// GL_ARB_uniform_buffer_object
    pub gl_arb_uniform_buffer_object: bool,
    /// GL_ARB_sync
    pub gl_arb_sync: bool,
    /// GL_ARB_get_program_binary
    pub gl_arb_get_programy_binary: bool,
    /// GL_ARB_tessellation_shader
    pub gl_arb_tessellation_shader: bool,
    /// GL_APPLE_vertex_array_object
    pub gl_apple_vertex_array_object: bool,
    /// GL_ARB_instanced_arrays
    pub gl_arb_instanced_arrays: bool,
    /// GL_ARB_vertex_buffer_object
    pub gl_arb_vertex_buffer_object: bool,
    /// GL_ARB_map_buffer_range
    pub gl_arb_map_buffer_range: bool,
    /// GL_ARB_shader_objects
    pub gl_arb_shader_objects: bool,
    /// GL_ARB_vertex_shader
    pub gl_arb_vertex_shader: bool,
    /// GL_ARB_fragment_shader
    pub gl_arb_fragment_shader: bool,
    /// GL_ARB_texture_rg
    pub gl_arb_texture_rg: bool,
    /// GL_EXT_texture_integer
    pub gl_ext_texture_integer: bool,
    /// GL_ARB_texture_rgb10_a2ui
    pub gl_arb_texture_rgb10_a2ui: bool,
    /// GL_ARB_texture_float
    pub gl_arb_texture_float: bool,
    /// GL_EXT_packed_depth_stencil
    pub gl_ext_packed_depth_stencil: bool,
    /// GL_ARB_debug_output
    pub gl_arb_debug_output: bool,
    /// GL_ARB_depth_texture
    pub gl_arb_depth_texture: bool,
    /// GL_ARB_invalidate_subdata
    pub gl_arb_invalidate_subdata: bool,
    /// GL_EXT_transform_feedback
    pub gl_ext_transform_feedback: bool,
    /// GL_EXT_gpu_shader4
    pub gl_ext_gpu_shader4: bool,
    /// GL_ARB_compute_shader
    pub gl_arb_compute_shader: bool,
    /// GL_ARB_geometry_shader4
    pub gl_arb_geometry_shader4: bool,
    /// GL_ARB_texture_non_power_of_two
    pub gl_arb_texture_non_power_of_two: bool,
    /// GL_ARB_texture_multisample
    pub gl_arb_texture_multisample: bool,
    /// GL_EXT_disjoint_timer_query
    pub gl_ext_disjoint_timer_query: bool,
}

pub fn get_extensions(gl: &gl::Gl) -> ExtensionsList {
    let strings = get_extensions_strings(gl);

    let mut extensions = ExtensionsList {
        gl_ext_direct_state_access: false,
        gl_arb_direct_state_access: false,
        gl_ext_framebuffer_object: false,
        gl_ext_geometry_shader4: false,
        gl_ext_framebuffer_blit: false,
        gl_khr_debug: false,
        gl_nvx_gpu_memory_info: false,
        gl_ati_meminfo: false,
        gl_arb_vertex_array_object: false,
        gl_arb_sampler_objects: false,
        gl_ext_texture_filter_anisotropic: false,
        gl_arb_texture_storage: false,
        gl_arb_buffer_storage: false,
        gl_arb_uniform_buffer_object: false,
        gl_arb_sync: false,
        gl_arb_get_programy_binary: false,
        gl_arb_tessellation_shader: false,
        gl_apple_vertex_array_object: false,
        gl_arb_instanced_arrays: false,
        gl_arb_vertex_buffer_object: false,
        gl_arb_map_buffer_range: false,
        gl_arb_shader_objects: false,
        gl_arb_vertex_shader: false,
        gl_arb_fragment_shader: false,
        gl_arb_texture_rg: false,
        gl_ext_texture_integer: false,
        gl_arb_texture_rgb10_a2ui: false,
        gl_arb_texture_float: false,
        gl_ext_packed_depth_stencil: false,
        gl_arb_debug_output: false,
        gl_arb_depth_texture: false,
        gl_arb_invalidate_subdata: false,
        gl_ext_transform_feedback: false,
        gl_ext_gpu_shader4: false,
        gl_arb_compute_shader: false,
        gl_arb_geometry_shader4: false,
        gl_arb_texture_non_power_of_two: false,
        gl_arb_texture_multisample: false,
        gl_ext_disjoint_timer_query: false,
    };

    for extension in strings.into_iter() {
        match extension.as_slice() {
            "GL_EXT_direct_state_access" => extensions.gl_ext_direct_state_access = true,
            "GL_ARB_direct_state_access" => extensions.gl_arb_direct_state_access = true,
            "GL_EXT_framebuffer_object" => extensions.gl_ext_framebuffer_object = true,
            "GL_EXT_geometry_shader4" => extensions.gl_ext_geometry_shader4 = true,
            "GL_EXT_framebuffer_blit" => extensions.gl_ext_framebuffer_blit = true,
            "GL_KHR_debug" => extensions.gl_khr_debug = true,
            "GL_NVX_gpu_memory_info" => extensions.gl_nvx_gpu_memory_info = true,
            "GL_ATI_meminfo" => extensions.gl_ati_meminfo = true,
            "GL_ARB_vertex_array_object" => extensions.gl_arb_vertex_array_object = true,
            "GL_ARB_sampler_objects" => extensions.gl_arb_sampler_objects = true,
            "GL_EXT_texture_filter_anisotropic" => extensions.gl_ext_texture_filter_anisotropic = true,
            "GL_ARB_texture_storage" => extensions.gl_arb_texture_storage = true,
            "GL_ARB_buffer_storage" => extensions.gl_arb_buffer_storage = true,
            "GL_ARB_uniform_buffer_object" => extensions.gl_arb_uniform_buffer_object = true,
            "GL_ARB_sync" => extensions.gl_arb_sync = true,
            "GL_ARB_get_program_binary" => extensions.gl_arb_get_programy_binary = true,
            "GL_ARB_tessellation_shader" => extensions.gl_arb_tessellation_shader = true,
            "GL_APPLE_vertex_array_object" => extensions.gl_apple_vertex_array_object = true,
            "GL_ARB_instanced_arrays" => extensions.gl_arb_instanced_arrays = true,
            "GL_ARB_vertex_buffer_object" => extensions.gl_arb_vertex_buffer_object = true,
            "GL_ARB_map_buffer_range" => extensions.gl_arb_map_buffer_range = true,
            "GL_ARB_shader_objects" => extensions.gl_arb_shader_objects = true,
            "GL_ARB_vertex_shader" => extensions.gl_arb_vertex_shader = true,
            "GL_ARB_fragment_shader" => extensions.gl_arb_fragment_shader = true,
            "GL_ARB_texture_rg" => extensions.gl_arb_texture_rg = true,
            "GL_EXT_texture_integer" => extensions.gl_ext_texture_integer = true,
            "GL_ARB_texture_rgb10_a2ui" => extensions.gl_arb_texture_rgb10_a2ui = true,
            "GL_ARB_texture_float" => extensions.gl_arb_texture_float = true,
            "GL_EXT_packed_depth_stencil" => extensions.gl_ext_packed_depth_stencil = true,
            "GL_ARB_debug_output" => extensions.gl_arb_debug_output = true,
            "GL_ARB_depth_texture" => extensions.gl_arb_depth_texture = true,
            "GL_ARB_invalidate_subdata" => extensions.gl_arb_invalidate_subdata = true,
            "GL_EXT_transform_feedback" => extensions.gl_ext_transform_feedback = true,
            "GL_EXT_gpu_shader4" => extensions.gl_ext_gpu_shader4 = true,
            "GL_ARB_compute_shader" => extensions.gl_arb_compute_shader = true,
            "GL_ARB_geometry_shader4" => extensions.gl_arb_geometry_shader4 = true,
            "GL_ARB_texture_non_power_of_two" => extensions.gl_arb_texture_non_power_of_two = true,
            "GL_ARB_texture_multisample" => extensions.gl_arb_texture_multisample = true,
            "GL_EXT_disjoint_timer_query" => extensions.gl_ext_disjoint_timer_query = true,
            _ => ()
        }
    }

    extensions
}

fn get_extensions_strings(gl: &gl::Gl) -> Vec<String> {
    unsafe {
        let list = gl.GetString(gl::EXTENSIONS);

        if list.is_null() {
            let mut num_extensions = 0;
            gl.GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);

            range(0, num_extensions).map(|num| {
                let ext = gl.GetStringi(gl::EXTENSIONS, num as gl::types::GLuint);
                String::from_utf8(ffi::c_str_to_bytes(&(ext as *const i8)).to_vec()).unwrap()
            }).collect()

        } else {
            let list = String::from_utf8(ffi::c_str_to_bytes(&(list as *const i8)).to_vec()).unwrap();
            list.words().map(|e| e.to_string()).collect()
        }
    }
}
