extern crate gl_generator;
extern crate khronos_api;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use gl_generator::generators::Generator;

mod textures;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&dest);

    textures::build_texture_file(&mut File::create(&dest.join("textures.rs")).unwrap());


    // There is a `#[derive(Clone)]` line in the bindings that triggers a stack overflow
    // in rustc (https://github.com/rust-lang/rust/issues/26467).
    // Therefore we write the bindings to memory first, then remove this line, and then copy
    // to the file.
    let mut gl_bindings = Vec::new();
    generate_gl_bindings(&mut gl_bindings);
    let gl_bindings = String::from_utf8(gl_bindings).unwrap();
    let gl_bindings = gl_bindings.replace("#[derive(Clone)]", "");
    let mut file_output = File::create(&dest.join("gl_bindings.rs")).unwrap();
    file_output.write_all(&gl_bindings.into_bytes()).unwrap();
}

fn generate_gl_bindings<W>(dest: &mut W) where W: Write {
    let gl_registry = {
        let reader = BufReader::new(khronos_api::GL_XML);
        let ns = gl_generator::registry::Ns::Gl;

        let filter = gl_generator::registry::Filter {
            fallbacks: gl_generator::Fallbacks::None,
            api: gl_generator::registry::Ns::Gl.to_string(),
            extensions: vec![
                "GL_AMD_depth_clamp_separate".to_string(),
                "GL_APPLE_vertex_array_object".to_string(),
                "GL_ARB_bindless_texture".to_string(),
                "GL_ARB_buffer_storage".to_string(),
                "GL_ARB_compute_shader".to_string(),
                "GL_ARB_copy_buffer".to_string(),
                "GL_ARB_debug_output".to_string(),
                "GL_ARB_depth_texture".to_string(),
                "GL_ARB_direct_state_access".to_string(),
                "GL_ARB_draw_buffers".to_string(),
                "GL_ARB_ES2_compatibility".to_string(),
                "GL_ARB_ES3_compatibility".to_string(),
                "GL_ARB_ES3_1_compatibility".to_string(),
                "GL_ARB_ES3_2_compatibility".to_string(),
                "GL_ARB_framebuffer_sRGB".to_string(),
                "GL_ARB_geometry_shader4".to_string(),
                "GL_ARB_gpu_shader_fp64".to_string(),
                "GL_ARB_gpu_shader_int64".to_string(),
                "GL_ARB_invalidate_subdata".to_string(),
                "GL_ARB_multi_draw_indirect".to_string(),
                "GL_ARB_occlusion_query".to_string(),
                "GL_ARB_pixel_buffer_object".to_string(),
                "GL_ARB_robustness".to_string(),
                "GL_ARB_shader_image_load_store".to_string(),
                "GL_ARB_shader_objects".to_string(),
                "GL_ARB_texture_buffer_object".to_string(),
                "GL_ARB_texture_float".to_string(),
                "GL_ARB_texture_multisample".to_string(),
                "GL_ARB_texture_rg".to_string(),
                "GL_ARB_texture_rgb10_a2ui".to_string(),
                "GL_ARB_transform_feedback3".to_string(),
                "GL_ARB_vertex_buffer_object".to_string(),
                "GL_ARB_vertex_shader".to_string(),
                "GL_ATI_draw_buffers".to_string(),
                "GL_ATI_meminfo".to_string(),
                "GL_EXT_debug_marker".to_string(),
                "GL_EXT_direct_state_access".to_string(),
                "GL_EXT_framebuffer_blit".to_string(),
                "GL_EXT_framebuffer_multisample".to_string(),
                "GL_EXT_framebuffer_object".to_string(),
                "GL_EXT_framebuffer_sRGB".to_string(),
                "GL_EXT_gpu_shader4".to_string(),
                "GL_EXT_packed_depth_stencil".to_string(),
                "GL_EXT_provoking_vertex".to_string(),
                "GL_EXT_texture_array".to_string(),
                "GL_EXT_texture_buffer_object".to_string(),
                "GL_EXT_texture_compression_s3tc".to_string(),
                "GL_EXT_texture_filter_anisotropic".to_string(),
                "GL_EXT_texture_integer".to_string(),
                "GL_EXT_texture_sRGB".to_string(),
                "GL_EXT_transform_feedback".to_string(),
                "GL_GREMEDY_string_marker".to_string(),
                "GL_KHR_robustness".to_string(),
                "GL_NVX_gpu_memory_info".to_string(),
                "GL_NV_conditional_render".to_string(),
                "GL_NV_vertex_attrib_integer_64bit".to_string(),
            ],
            version: "4.5".to_string(),
            profile: "compatibility".to_string(),
        };

        gl_generator::registry::Registry::from_xml(reader, ns, Some(filter))
    };

    let gles_registry = {
        let reader = BufReader::new(khronos_api::GL_XML);
        let ns = gl_generator::registry::Ns::Gles2;

        let filter = gl_generator::registry::Filter {
            fallbacks: gl_generator::Fallbacks::None,
            api: gl_generator::registry::Ns::Gles2.to_string(),
            extensions: vec![
                "GL_ANGLE_framebuffer_multisample".to_string(),
                "GL_APPLE_framebuffer_multisample".to_string(),
                "GL_APPLE_sync".to_string(),
                "GL_ARM_rgba8".to_string(),
                "GL_EXT_buffer_storage".to_string(),
                "GL_EXT_disjoint_timer_query".to_string(),
                "GL_EXT_multi_draw_indirect".to_string(),
                "GL_EXT_multisampled_render_to_texture".to_string(),
                "GL_EXT_occlusion_query_boolean".to_string(),
                "GL_EXT_primitive_bounding_box".to_string(),
                "GL_EXT_robustness".to_string(),
                "GL_KHR_debug".to_string(),
                "GL_NV_copy_buffer".to_string(),
                "GL_NV_framebuffer_multisample".to_string(),
                "GL_NV_pixel_buffer_object".to_string(),
                "GL_OES_depth_texture".to_string(),
                "GL_OES_draw_elements_base_vertex".to_string(),
                "GL_OES_packed_depth_stencil".to_string(),
                "GL_OES_primitive_bounding_box".to_string(),
                "GL_OES_rgb8_rgba8".to_string(),
                "GL_OES_texture_buffer".to_string(),
                "GL_OES_texture_npot".to_string(),
                "GL_OES_vertex_array_object".to_string(),
                "GL_OES_vertex_type_10_10_10_2".to_string(),
            ],
            version: "3.2".to_string(),
            profile: "compatibility".to_string(),
        };

        gl_generator::registry::Registry::from_xml(reader, ns, Some(filter))
    };

    gl_generator::StructGenerator.write(&(gl_registry + gles_registry),
                                        gl_generator::registry::Ns::Gl, dest).unwrap();
}
