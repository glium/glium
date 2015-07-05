#[macro_use]
extern crate glium;
use glium::glutin;

fn main() {
    use glium::DisplayBuild;

    let display = glutin::HeadlessRendererBuilder::new(1024, 1024)
        .build_glium()
        .unwrap();

    let program = glium::program::ComputeShader::from_source(&display, r#"\

            #version 430
            buffer layout(shared);
            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            buffer MyBlock {
                ivec4 value[256];
            };

            void main() {
                ivec4 val = value[gl_GlobalInvocationID.x];
                value[gl_GlobalInvocationID.x] = val * val;
            }

        "#).unwrap();

    #[derive(Copy)]
    struct Data {
        value: [i32; 1024],
    }

    impl Clone for Data {
        fn clone(&self) -> Data {
            *self
        }
    }

    implement_uniform_block!(Data, value);

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
                            glium::uniforms::UniformBuffer::empty_if_supported(&display).unwrap();

    {
        let mut mapping = buffer.map();
        for (id, v) in mapping.value.iter_mut().enumerate() {
            *v = id as i32;
        }
    }

    program.execute(uniform! { MyBlock: &buffer }, 1024, 1, 1);

    {
        let mapping = buffer.map();
        assert_eq!(mapping.value[0], 0);
        assert_eq!(mapping.value[1], 1);
        assert_eq!(mapping.value[2], 4);
        assert_eq!(mapping.value[3], 9);
        assert_eq!(mapping.value[4], 16);
        assert_eq!(mapping.value[5], 25);
        assert_eq!(mapping.value[6], 36);
    }
}
