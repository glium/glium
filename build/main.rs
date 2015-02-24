extern crate gl_generator;
extern crate khronos_api;

use std::env;
use std::old_io::File;
use std::old_io::BufReader;
use gl_generator::generators::Generator;

mod textures;

fn main() {
    let dest = Path::new(env::var("OUT_DIR").unwrap());

    textures::build_texture_file(&mut File::create(&dest.join("textures.rs")).unwrap());


    let mut gl_bindings = File::create(&dest.join("gl_bindings.rs")).unwrap();
    generate_gl_bindings(&mut gl_bindings);
}

fn generate_gl_bindings<W>(dest: &mut W) where W: Writer {
    let gl_registry = {
        let reader = BufReader::new(khronos_api::GL_XML);
        let ns = gl_generator::registry::Ns::Gl;

        let filter = gl_generator::registry::Filter {
            api: gl_generator::registry::Ns::Gl.to_string(),
            extensions: vec![
                "GL_EXT_direct_state_access".to_string(),
                "GL_ARB_direct_state_access".to_string(),
                "GL_EXT_framebuffer_object".to_string(),
                "GL_EXT_framebuffer_blit".to_string(),
                "GL_NVX_gpu_memory_info".to_string(),
                "GL_ATI_meminfo".to_string(),
                "GL_EXT_texture_filter_anisotropic".to_string(),
                "GL_ARB_buffer_storage".to_string(),
                "GL_APPLE_vertex_array_object".to_string(),
                "GL_ARB_vertex_buffer_object".to_string(),
                "GL_ARB_shader_objects".to_string(),
                "GL_ARB_vertex_shader".to_string(),
                "GL_ARB_texture_rg".to_string(),
                "GL_EXT_texture_integer".to_string(),
                "GL_ARB_texture_rgb10_a2ui".to_string(),
                "GL_ARB_texture_float".to_string(),
                "GL_EXT_packed_depth_stencil".to_string(),
                "GL_ARB_debug_output".to_string(),
                "GL_ARB_depth_texture".to_string(),
                "GL_ARB_invalidate_subdata".to_string(),
                "GL_EXT_transform_feedback".to_string(),
                "GL_EXT_gpu_shader4".to_string(),
                "GL_ARB_compute_shader".to_string(),
                "GL_ARB_geometry_shader4".to_string(),
                "GL_ARB_texture_multisample".to_string(),
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
            api: gl_generator::registry::Ns::Gles2.to_string(),
            extensions: vec![
                "GL_OES_texture_npot".to_string(),
                "GL_EXT_disjoint_timer_query".to_string(),
            ],
            version: "3.1".to_string(),
            profile: "compatibility".to_string(),
        };

        gl_generator::registry::Registry::from_xml(reader, ns, Some(filter))
    };

    gl_generator::StructGenerator.write(&(gl_registry + gles_registry),
                                        gl_generator::registry::Ns::Gl, dest).unwrap();
}
