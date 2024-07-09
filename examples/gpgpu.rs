#[macro_use]
extern crate glium;
mod support;

use glium::{Display};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

struct Application { }

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium GPGPU example";

    fn new(display: &Display<WindowSurface>) -> Self {
        let program = glium::program::ComputeShader::from_source(display, r#"\
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

        const NUM_VALUES: usize = 4096;

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct Data {
            power: f32,
            _padding: [f32; 3],
            values: [[f32; 4]; NUM_VALUES / 4],
        }
        implement_uniform_block!(Data, power, values);

        let mut buffer: glium::uniforms::UniformBuffer<Data> =
                glium::uniforms::UniformBuffer::empty(display).unwrap();

        {
            let mut mapping = buffer.map();
            mapping.power = rand::random();
            for val in mapping.values.iter_mut() {
                *val = [rand::random::<f32>(),rand::random::<f32>(),rand::random::<f32>(),rand::random::<f32>()];
            }
        }

        program.execute(uniform! { MyBlock: &*buffer }, NUM_VALUES as u32 / 4, 1, 1);

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

        Self { }
    }
}

fn main() {
    State::<Application>::run_once(false);
}
