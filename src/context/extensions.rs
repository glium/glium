use std::ffi::CStr;
use version::Version;
use version::Api;
use gl;

macro_rules! extensions {
    ($($string:expr => $field:ident,)+) => {
        /// Contains data about the list of extensions.
        #[derive(Debug, Clone, Copy)]
        pub struct ExtensionsList {
            $(
                pub $field: bool,
            )+
        }

        /// Returns the list of extensions supported by the backend.
        ///
        /// The version must match the one of the backend.
        ///
        /// *Safety*: the OpenGL context corresponding to `gl` must be current in the thread.
        ///
        /// ## Panic
        ///
        /// Can panic if the version number doesn't match the backend, leading to unloaded functions
        /// being called.
        ///
        pub unsafe fn get_extensions(gl: &gl::Gl, version: &Version) -> ExtensionsList {
            let strings = get_extensions_strings(gl, version);

            let mut extensions = ExtensionsList {
                $(
                    $field: false,
                )+
            };

            for extension in strings.into_iter() {
                match &extension[..] {
                    $(
                        $string => extensions.$field = true,
                    )+
                    _ => ()
                }
            }

            extensions
        }
    }
}

extensions! {
    "GL_AMD_depth_clamp_separate" => gl_amd_depth_clamp_separate,
    "GL_AMD_query_buffer_object" => gl_amd_query_buffer_object,
    "GL_ANGLE_framebuffer_multisample" => gl_angle_framebuffer_multisample,
    "GL_APPLE_framebuffer_multisample" => gl_apple_framebuffer_multisample,
    "GL_APPLE_sync" => gl_apple_sync,
    "GL_APPLE_vertex_array_object" => gl_apple_vertex_array_object,
    "GL_ARB_bindless_texture" => gl_arb_bindless_texture,
    "GL_ARB_buffer_storage" => gl_arb_buffer_storage,
    "GL_ARB_compute_shader" => gl_arb_compute_shader,
    "GL_ARB_copy_buffer" => gl_arb_copy_buffer,
    "GL_ARB_debug_output" => gl_arb_debug_output,
    "GL_ARB_depth_clamp" => gl_arb_depth_clamp,
    "GL_ARB_depth_texture" => gl_arb_depth_texture,
    "GL_ARB_direct_state_access" => gl_arb_direct_state_access,
    "GL_ARB_draw_buffers" => gl_arb_draw_buffers,
    "GL_ARB_draw_elements_base_vertex" => gl_arb_draw_elements_base_vertex,
    "GL_ARB_compatibility" => gl_arb_compatibility,
    "GL_ARB_ES2_compatibility" => gl_arb_es2_compatibility,
    "GL_ARB_ES3_compatibility" => gl_arb_es3_compatibility,
    "GL_ARB_ES3_1_compatibility" => gl_arb_es3_1_compatibility,
    "GL_ARB_ES3_2_compatibility" => gl_arb_es3_2_compatibility,
    "GL_ARB_fragment_shader" => gl_arb_fragment_shader,
    "GL_ARB_framebuffer_no_attachments" => gl_arb_framebuffer_no_attachments,
    "GL_ARB_framebuffer_object" => gl_arb_framebuffer_object,
    "GL_ARB_framebuffer_sRGB" => gl_arb_framebuffer_srgb,
    "GL_ARB_geometry_shader4" => gl_arb_geometry_shader4,
    "GL_ARB_get_program_binary" => gl_arb_get_programy_binary,
    "GL_ARB_gpu_shader_fp64" => gl_arb_gpu_shader_fp64,
    "GL_ARB_gpu_shader_int64" => gl_arb_gpu_shader_int64,
    "GL_ARB_instanced_arrays" => gl_arb_instanced_arrays,
    "GL_ARB_internalformat_query" => gl_arb_internalformat_query,
    "GL_ARB_invalidate_subdata" => gl_arb_invalidate_subdata,
    "GL_ARB_occlusion_query" => gl_arb_occlusion_query,
    "GL_ARB_occlusion_query2" => gl_arb_occlusion_query2,
    "GL_ARB_pixel_buffer_object" => gl_arb_pixel_buffer_object,
    "GL_ARB_program_interface_query" => gl_arb_program_interface_query,
    "GL_ARB_query_buffer_object" => gl_arb_query_buffer_object,
    "GL_ARB_map_buffer_range" => gl_arb_map_buffer_range,
    "GL_ARB_multi_draw_indirect" => gl_arb_multi_draw_indirect,
    "GL_ARB_provoking_vertex" => gl_arb_provoking_vertex,
    "GL_ARB_robustness" => gl_arb_robustness,
    "GL_ARB_robust_buffer_access_behavior" => gl_arb_robust_buffer_access_behavior,
    "GL_ARB_sampler_objects" => gl_arb_sampler_objects,
    "GL_ARB_shader_image_load_store" => gl_arb_shader_image_load_store,
    "GL_ARB_shader_objects" => gl_arb_shader_objects,
    "GL_ARB_shader_storage_buffer_object" => gl_arb_shader_storage_buffer_object,
    "GL_ARB_shader_subroutine" => gl_arb_shader_subroutine,
    "GL_ARB_sync" => gl_arb_sync,
    "GL_ARB_tessellation_shader" => gl_arb_tessellation_shader,
    "GL_ARB_texture_buffer_object" => gl_arb_texture_buffer_object,
    "GL_ARB_texture_buffer_object_rgb32" => gl_arb_texture_buffer_object_rgb32,
    "GL_ARB_texture_compression_bptc" => gl_arb_texture_compression_bptc,
    "GL_ARB_texture_cube_map" => gl_arb_texture_cube_map,
    "GL_ARB_texture_cube_map_array" => gl_arb_texture_cube_map_array,
    "GL_ARB_texture_float" => gl_arb_texture_float,
    "GL_ARB_texture_multisample" => gl_arb_texture_multisample,
    "GL_ARB_texture_non_power_of_two" => gl_arb_texture_non_power_of_two,
    "GL_ARB_texture_rg" => gl_arb_texture_rg,
    "GL_ARB_texture_rgb10_a2ui" => gl_arb_texture_rgb10_a2ui,
    "GL_ARB_texture_stencil8" => gl_arb_texture_stencil8,
    "GL_ARB_texture_storage" => gl_arb_texture_storage,
    "GL_ARB_timer_query" => gl_arb_timer_query,
    "GL_ARB_transform_feedback3" => gl_arb_transform_feedback3,
    "GL_ARB_uniform_buffer_object" => gl_arb_uniform_buffer_object,
    "GL_ARB_vertex_array_object" => gl_arb_vertex_array_object,
    "GL_ARB_vertex_buffer_object" => gl_arb_vertex_buffer_object,
    "GL_ARB_vertex_half_float" => gl_arb_vertex_half_float,
    "GL_ARB_vertex_shader" => gl_arb_vertex_shader,
    "GL_ARB_vertex_type_10f_11f_11f_rev" => gl_arb_vertex_type_10f_11f_11f_rev,
    "GL_ARB_vertex_type_2_10_10_10_rev" => gl_arb_vertex_type_2_10_10_10_rev,
    "GL_ARM_rgba8" => gl_arm_rgba8,
    "GL_ATI_meminfo" => gl_ati_meminfo,
    "GL_ATI_draw_buffers" => gl_ati_draw_buffers,
    "GL_ATI_texture_float" => gl_ati_texture_float,
    "GL_EXT_blend_minmax" => gl_ext_blend_minmax,
    "GL_EXT_buffer_storage" => gl_ext_buffer_storage,
    "GL_EXT_debug_marker" => gl_ext_debug_marker,
    "GL_EXT_direct_state_access" => gl_ext_direct_state_access,
    "GL_EXT_disjoint_timer_query" => gl_ext_disjoint_timer_query,
    "GL_EXT_framebuffer_blit" => gl_ext_framebuffer_blit,
    "GL_EXT_framebuffer_object" => gl_ext_framebuffer_object,
    "GL_EXT_framebuffer_multisample" => gl_ext_framebuffer_multisample,
    "GL_EXT_framebuffer_sRGB" => gl_ext_framebuffer_srgb,
    "GL_EXT_geometry_shader" => gl_ext_geometry_shader,
    "GL_EXT_geometry_shader4" => gl_ext_geometry_shader4,
    "GL_EXT_gpu_shader4" => gl_ext_gpu_shader4,
    "GL_EXT_multi_draw_indirect" => gl_ext_multi_draw_indirect,
    "GL_EXT_multisampled_render_to_texture" => gl_ext_multisampled_render_to_texture,
    "GL_EXT_occlusion_query_boolean" => gl_ext_occlusion_query_boolean,
    "GL_EXT_packed_depth_stencil" => gl_ext_packed_depth_stencil,
    "GL_EXT_packed_float" => gl_ext_packed_float,
    "GL_EXT_primitive_bounding_box" => gl_ext_primitive_bounding_box,
    "GL_EXT_provoking_vertex" => gl_ext_provoking_vertex,
    "GL_EXT_robustness" => gl_ext_robustness,
    "GL_EXT_sRGB_write_control" => gl_ext_srgb_write_control,
    "GL_EXT_texture3D" => gl_ext_texture3d,
    "GL_EXT_texture_array" => gl_ext_texture_array,
    "GL_EXT_texture_buffer" => gl_ext_texture_buffer,
    "GL_EXT_texture_buffer_object" => gl_ext_texture_buffer_object,
    "GL_EXT_texture_compression_s3tc" => gl_ext_texture_compression_s3tc,
    "GL_EXT_texture_cube_map" => gl_ext_texture_cube_map,
    "GL_EXT_texture_cube_map_array" => gl_ext_texture_cube_map_array,
    "GL_EXT_texture_filter_anisotropic" => gl_ext_texture_filter_anisotropic,
    "GL_EXT_texture_integer" => gl_ext_texture_integer,
    "GL_EXT_texture_shared_exponent" => gl_ext_texture_shared_exponent,
    "GL_EXT_texture_snorm" => gl_ext_texture_snorm,
    "GL_EXT_texture_sRGB" => gl_ext_texture_srgb,
    "GL_EXT_transform_feedback" => gl_ext_transform_feedback,
    "GL_GREMEDY_string_marker" => gl_gremedy_string_marker,
    "GL_KHR_debug" => gl_khr_debug,
    "GL_KHR_context_flush_control" => gl_khr_context_flush_control,
    "GL_KHR_robustness" => gl_khr_robustness,
    "GL_KHR_robust_buffer_access_behavior" => gl_khr_robust_buffer_access_behavior,
    "GL_NV_fbo_color_attachments" => gl_nv_fbo_color_attachments,
    "GL_NV_conditional_render" => gl_nv_conditional_render,
    "GL_NV_copy_buffer" => gl_nv_copy_buffer,
    "GL_NV_depth_clamp" => gl_nv_depth_clamp,
    "GL_NV_framebuffer_multisample" => gl_nv_framebuffer_multisample,
    "GL_NV_half_float" => gl_nv_half_float,
    "GL_NV_internalformat_sample_query" => gl_nv_internalformat_sample_query,
    "GL_NV_pixel_buffer_object" => gl_nv_pixel_buffer_object,
    "GL_NV_read_depth" => gl_nv_read_depth,
    "GL_NV_read_stencil" => gl_nv_read_stencil,
    "GL_NV_read_depth_stencil" => gl_nv_read_depth_stencil,
    "GL_NV_texture_array" => gl_nv_texture_array,
    "GL_NV_vertex_attrib_integer_64bit" => gl_nv_vertex_attrib_integer_64bit,
    "GL_NVX_gpu_memory_info" => gl_nvx_gpu_memory_info,
    "GL_OES_depth_texture" => gl_oes_depth_texture,
    "GL_OES_draw_elements_base_vertex" => gl_oes_draw_elements_base_vertex,
    "GL_OES_element_index_uint" => gl_oes_element_index_uint,
    "GL_OES_fixed_point" => gl_oes_fixed_point,
    "GL_OES_geometry_shader" => gl_oes_geometry_shader,
    "GL_OES_packed_depth_stencil" => gl_oes_packed_depth_stencil,
    "GL_OES_primitive_bounding_box" => gl_oes_primitive_bounding_box,
    "GL_OES_rgb8_rgba8" => gl_oes_rgb8_rgba8,
    "GL_OES_stencil1" => gl_oes_stencil1,
    "GL_OES_stencil4" => gl_oes_stencil4,
    "GL_OES_tessellation_shader" => gl_oes_tessellation_shader,
    "GL_OES_texture_3D" => gl_oes_texture_3d,
    "GL_OES_texture_buffer" => gl_oes_texture_buffer,
    "GL_OES_texture_cube_map_array" => gl_oes_texture_cube_map_array,
    "GL_OES_texture_stencil8" => gl_oes_texture_stencil8,
    "GL_OES_texture_storage_multisample_2d_array" => gl_oes_texture_storage_multisample_2d_array,
    "GL_OES_vertex_array_object" => gl_oes_vertex_array_object,
    "GL_OES_vertex_half_float" => gl_oes_vertex_half_float,
    "GL_OES_vertex_type_10_10_10_2" => gl_oes_vertex_type_10_10_10_2,
}

/// Returns the list of all extension names supported by the OpenGL implementation.
///
/// The version must match the one of the backend.
///
/// *Safety*: the OpenGL context corresponding to `gl` must be current in the thread.
///
/// ## Panic
///
/// Can panic if the version number doesn't match the backend, leading to unloaded functions
/// being called.
///
unsafe fn get_extensions_strings(gl: &gl::Gl, version: &Version) -> Vec<String> {
    if version >= &Version(Api::Gl, 3, 0) || version >= &Version(Api::GlEs, 3, 0) {
        let mut num_extensions = 0;
        gl.GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);

        (0 .. num_extensions).map(|num| {
            let ext = gl.GetStringi(gl::EXTENSIONS, num as gl::types::GLuint);
            String::from_utf8(CStr::from_ptr(ext as *const _).to_bytes().to_vec()).unwrap()
        }).collect()

    } else {
        let list = gl.GetString(gl::EXTENSIONS);
        assert!(!list.is_null());
        let list = String::from_utf8(CStr::from_ptr(list as *const _).to_bytes().to_vec())
                                     .unwrap();
        list.split(' ').map(|e| e.to_owned()).collect()
    }
}
