extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod textures;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&dest);

    textures::build_texture_file(&mut File::create(&dest.join("textures.rs")).unwrap());
    println!("cargo:rerun-if-changed=build/main.rs");


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
    let gl_registry = Registry::new(
        Api::Gl,
        (4, 5),
        Profile::Compatibility,
        Fallbacks::None,
        vec![
            "GL_AMD_depth_clamp_separate",
            "GL_APPLE_vertex_array_object",
            "GL_ARB_bindless_texture",
            "GL_ARB_buffer_storage",
            "GL_ARB_compute_shader",
            "GL_ARB_copy_buffer",
            "GL_ARB_debug_output",
            "GL_ARB_depth_texture",
            "GL_ARB_direct_state_access",
            "GL_ARB_draw_buffers",
            "GL_ARB_ES2_compatibility",
            "GL_ARB_ES3_compatibility",
            "GL_ARB_ES3_1_compatibility",
            "GL_ARB_ES3_2_compatibility",
            "GL_ARB_framebuffer_sRGB",
            "GL_ARB_geometry_shader4",
            "GL_ARB_gpu_shader_fp64",
            "GL_ARB_gpu_shader_int64",
            "GL_ARB_invalidate_subdata",
            "GL_ARB_multi_draw_indirect",
            "GL_ARB_occlusion_query",
            "GL_ARB_pixel_buffer_object",
            "GL_ARB_robustness",
            "GL_ARB_shader_image_load_store",
            "GL_ARB_shader_objects",
            "GL_ARB_texture_buffer_object",
            "GL_ARB_texture_float",
            "GL_ARB_texture_multisample",
            "GL_ARB_texture_rg",
            "GL_ARB_texture_rgb10_a2ui",
            "GL_ARB_transform_feedback3",
            "GL_ARB_vertex_buffer_object",
            "GL_ARB_vertex_shader",
            "GL_ATI_draw_buffers",
            "GL_ATI_meminfo",
            "GL_EXT_debug_marker",
            "GL_EXT_direct_state_access",
            "GL_EXT_framebuffer_blit",
            "GL_EXT_framebuffer_multisample",
            "GL_EXT_framebuffer_object",
            "GL_EXT_framebuffer_sRGB",
            "GL_EXT_gpu_shader4",
            "GL_EXT_packed_depth_stencil",
            "GL_EXT_provoking_vertex",
            "GL_EXT_texture_array",
            "GL_EXT_texture_buffer_object",
            "GL_EXT_texture_compression_s3tc",
            "GL_EXT_texture_filter_anisotropic",
            "GL_EXT_texture_integer",
            "GL_EXT_texture_sRGB",
            "GL_EXT_transform_feedback",
            "GL_GREMEDY_string_marker",
            "GL_KHR_robustness",
            "GL_NVX_gpu_memory_info",
            "GL_NV_conditional_render",
            "GL_NV_vertex_attrib_integer_64bit",
        ],
    );

    let gles_registry = Registry::new(
        Api::Gles2,
        (3, 2),
        Profile::Compatibility,
        Fallbacks::None,
        vec![
            "GL_ANGLE_framebuffer_multisample",
            "GL_APPLE_framebuffer_multisample",
            "GL_APPLE_sync",
            "GL_ARM_rgba8",
            "GL_EXT_buffer_storage",
            "GL_EXT_disjoint_timer_query",
            "GL_EXT_multi_draw_indirect",
            "GL_EXT_multisampled_render_to_texture",
            "GL_EXT_occlusion_query_boolean",
            "GL_EXT_primitive_bounding_box",
            "GL_EXT_robustness",
            "GL_KHR_debug",
            "GL_NV_copy_buffer",
            "GL_NV_framebuffer_multisample",
            "GL_NV_internalformat_sample_query",
            "GL_NV_pixel_buffer_object",
            "GL_OES_depth_texture",
            "GL_OES_draw_elements_base_vertex",
            "GL_OES_packed_depth_stencil",
            "GL_OES_primitive_bounding_box",
            "GL_OES_rgb8_rgba8",
            "GL_OES_texture_buffer",
            "GL_OES_texture_npot",
            "GL_OES_vertex_array_object",
            "GL_OES_vertex_type_10_10_10_2",
        ],
    );

    (gl_registry + gles_registry)
        .write_bindings(gl_generator::StructGenerator, dest)
        .unwrap();
}
