#[macro_use]
extern crate glium;
extern crate rand;
use glium::glutin;

fn main() {
    use glium::DisplayBuild;

    let display = glutin::HeadlessRendererBuilder::new(1024, 1024)
        .build_glium()
        .unwrap();

    let program = glium::program::ComputeShader::from_source(&display, r#"\

            #version 430
            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            layout(std140) buffer MyBlock {
                float power;
                float values[256];
            };

            void main() {
                float val = values[gl_GlobalInvocationID.x];
                values[gl_GlobalInvocationID.x] = pow(val, power);
            }

        "#).unwrap();

    struct Data {
        power: f32,
        _padding: [f32; 3],
        values: [f32],
    }

    implement_buffer_content!(Data);
    implement_uniform_block!(Data, power, values);

    const NUM_VALUES: usize = 4096;

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
              glium::uniforms::UniformBuffer::empty_unsized(&display, 4 + 4 * NUM_VALUES).unwrap();

    {
        let mut mapping = buffer.map();
        mapping.power = rand::random();
        for val in mapping.values.iter_mut() {
            *val = rand::random();
        }
    }

    program.execute(uniform! { MyBlock: &*buffer }, 4096, 1, 1);

    {
        let mapping = buffer.map();
        println!("Power is: {:?}", mapping.power);
        for val in mapping.values.iter().take(10) {
            println!("{:?}", *val);
        }
        println!("...");
    }
}
