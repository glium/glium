#[macro_use]
extern crate glium;
extern crate rand;
use glium::glutin;

fn main() {
    let event_loop = glium::glutin::EventsLoop::new();
    let context_builder = glutin::ContextBuilder::new();
    let context = glutin::Context::new(&event_loop, context_builder, false).unwrap();
    let display = glium::backend::glutin::headless::Headless::new(context).unwrap();

    let program = glium::program::ComputeShader::from_source(&display, r#"\

            #version 430
            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            layout(std140) buffer MyBlock {
                float power;
                vec4 values[4096/4];
            };

            void main() {
                vec4 val = values[gl_GlobalInvocationID.x];

                values[gl_GlobalInvocationID.x] = pow(val, vec4(power));
            }

        "#).unwrap();

    struct Data {
        power: f32,
        _padding: [f32; 3],
        values: [[f32;4]],
    }

    implement_buffer_content!(Data);
    implement_uniform_block!(Data, power, values);

    const NUM_VALUES: usize = 4096;

    let mut buffer: glium::uniforms::UniformBuffer<Data> =
              glium::uniforms::UniformBuffer::empty_unsized(&display, 4 + 4*3 + 4 * NUM_VALUES).unwrap();

    {
        let mut mapping = buffer.map();
        mapping.power = rand::random();
        for val in mapping.values.iter_mut() {
            *val = [rand::random::<f32>(),rand::random::<f32>(),rand::random::<f32>(),rand::random::<f32>()];
        }
    }

    program.execute(uniform! { MyBlock: &*buffer }, 4096, 1, 1);

    {
        let mapping = buffer.map();
        println!("Power is: {:?}", mapping.power);
        for val in mapping.values.iter().take(3) {
            println!("{:?}", val[0]);
            println!("{:?}", val[1]);
            println!("{:?}", val[2]);
            println!("{:?}", val[3]);
        }
        println!("...");
    }
}
