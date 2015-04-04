#![feature(plugin)]
#![feature(custom_attribute)]
#![plugin(glium_macros)]

extern crate glium;

#[test]
fn verify_shader() {
    #[vertex_format]
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2]
    }

    //assert_eq!(<Vertex as >)
}
