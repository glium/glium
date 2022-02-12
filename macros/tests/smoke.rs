// #[cfg(feature = "glslang")]
// #[test]
// fn verify_shader() {
//     static VERTEX_SHADER: &'static str = verify_shader!(vertex "
//         #version 110

//         void main() {
//             gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
//         }
//     ");
// }

#[test]
fn derive_vertex() {
    #[derive(glium_macros::Vertex, Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
}

#[test]
fn derive_uniform() {
    #[derive(glium_macros::Uniform, Copy, Clone)]
    struct Uniform {
        position: [f32; 2],
    }
}
