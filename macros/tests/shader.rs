#![cfg(feature = "glslang")]
#![feature(plugin)]

#[plugin]
extern crate glium_macros;

#[test]
fn verify_shader() {
    static VERTEX_SHADER: &'static str = verify_shader!(vertex "
        #version 110

        void main() {
            gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        }
    ");
}
