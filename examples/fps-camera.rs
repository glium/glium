#[macro_use]
extern crate glium;

#[path = "../book/tuto-07-teapot.rs"]
mod teapot;

use std::f32::consts::PI;

const UP: [f32; 3] = [0.0, 1.0, 0.0];

struct Camera {
    pub position: [f32; 3],
    pub phi: f32, // horizontal angle relative to -z, increasing to the right
    pub theta: f32, // inclination, relative to the xz-plane, increasing to the top

    pub moving_up: bool,
    pub moving_left: bool,
    pub moving_down: bool,
    pub moving_right: bool,
    pub moving_forward: bool,
    pub moving_backward: bool,
    pub turning_up: bool,
    pub turning_left: bool,
    pub turning_down: bool,
    pub turning_right: bool,
}

impl Camera {
    pub fn new(pos: [f32;3], phi: f32, theta: f32) -> Camera {
        let mut cam = Camera {
            position: pos,
            phi: phi,
            theta: theta,

            moving_up: false,
            moving_left: false,
            moving_down: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
            turning_up: false,
            turning_left: false,
            turning_down: false,
            turning_right: false,
        };
        cam.norm_phi();
        cam.norm_theta();
        cam
    }

    pub fn at(pos: [f32; 3], target: [f32; 3]) -> Camera {
        let dir: [f32; 3] = [
            target[0] - pos[0],
            target[1] - pos[1],
            target[2] - pos[2],
        ];

        let plane: [f32; 3] = [dir[0], 0.0, dir[2]];

        let phi = {
            let dot = plane[0] * 0.0 + plane[1] * 0.0 + plane[2] * -1.0;
            let len = plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2];
            let len = len.sqrt();
            let phi = dot / len;
            let phi = phi.acos();
            phi * plane[0].signum()
        };

        let theta = {
            let dot = dir[0] * plane[0] + dir[1] * plane[1] + dir[2] * plane[2];
            let len1 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
            let len1 = len1.sqrt();
            let len2 = plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2];
            let len2 = len2.sqrt();
            let theta = dot / (len1 * len2);
            let theta = theta.acos();
            theta * dir[1].signum()
        };

        Camera::new(pos, phi, theta)
    }

    fn norm_phi(&mut self) {
        self.phi += PI * 2.0;
        self.phi %= PI * 2.0;
    }

    fn norm_theta(&mut self) {
        self.theta = self.theta.max(-PI * 0.4999).min(PI * 0.4999);
    }

    pub fn direction(&self) -> [f32; 3] {
        [
            self.theta.cos() * self.phi.sin(),
            self.theta.sin(),
            -self.theta.cos() * self.phi.cos(),
        ]
    }

    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let f = {
            let f = self.direction();
            let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
            let len = len.sqrt();
            [f[0] / len, f[1] / len, f[2] / len]
        };

        let s = {
            let s = [
                f[1] * UP[2] - f[2] * UP[1],
                f[2] * UP[0] - f[0] * UP[2],
                f[0] * UP[1] - f[1] * UP[0],
            ];
            let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
            let len = len.sqrt();
            [s[0] / len, s[1] / len, s[2] / len]
        };

        let u = [
            s[1] * f[2] - s[2] * f[1],
            s[2] * f[0] - s[0] * f[2],
            s[0] * f[1] - s[1] * f[0],
        ];

        let p = [
            -(self.position[0] * s[0] + self.position[1] * s[1] + self.position[2] * s[2]),
            -(self.position[0] * u[0] + self.position[1] * u[1] + self.position[2] * u[2]),
            -(self.position[0] * f[0] + self.position[1] * f[1] + self.position[2] * f[2]),
        ];

        [
            [s[0], u[0], f[0], 0.0],
            [s[1], u[1], f[1], 0.0],
            [s[2], u[2], f[2], 0.0],
            [p[0], p[1], p[2], 1.0],
        ]
    }

    pub fn update(&mut self) {
        if self.moving_up {
            self.position[1] += 0.05;
        }

        if self.moving_left {
            let a = self.phi + PI * 1.5;
            self.position[0] += a.sin() * 0.05;
            self.position[2] -= a.cos() * 0.05;
        }

        if self.moving_down {
            self.position[1] -= 0.05;
        }

        if self.moving_right {
            let a = self.phi + PI * 0.5;
            self.position[0] += a.sin() * 0.05;
            self.position[2] -= a.cos() * 0.05;
        }

        if self.moving_forward {
            let a = self.phi + PI * 0.0;
            self.position[0] += a.sin() * 0.05;
            self.position[2] -= a.cos() * 0.05;
        }

        if self.moving_backward {
            let a = self.phi + PI * 1.0;
            self.position[0] += a.sin() * 0.05;
            self.position[2] -= a.cos() * 0.05;
        }
        if self.turning_up {
            self.theta += PI / 360.0;
        }
        if self.turning_left {
            self.phi -= PI / 360.0;
        }
        if self.turning_down {
            self.theta -= PI / 360.0;
        }
        if self.turning_right {
            self.phi += PI / 360.0;
        }
        self.norm_phi();
        self.norm_theta();
    }
}

fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new()
                        .with_depth_buffer(24)
                        .build_glium().unwrap();

    let window = display.get_window().unwrap();

    window.set_cursor(glium::glutin::MouseCursor::Crosshair);

    let (width, height) = window.get_inner_size_points().unwrap();
    let (width, height) = (width as i32 / 2, height as i32 / 2);
    window.set_cursor_position(width, height).unwrap();

    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                          &teapot::INDICES).unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        out vec4 color;
        uniform vec3 u_light;

        void main() {
            float brightness = dot(normalize(v_normal), normalize(u_light));
            vec3 dark_color = vec3(0.6, 0.0, 0.0);
            vec3 regular_color = vec3(1.0, 0.0, 0.0);
            color = vec4(mix(dark_color, regular_color, brightness), 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src,
                                              None).unwrap();
    let mut camera = Camera::at([0.0, 0.0, 2.0], [0.0, 0.0, 0.0]);

    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let model: [[f32; 4]; 4] = [
            [0.01, 0.0 , 0.0 , 0.0],
            [0.0 , 0.01, 0.0 , 0.0],
            [0.0 , 0.0 , 0.01, 0.0],
            [0.0 , 0.0 , 0.0 , 1.0],
        ];

        let view = camera.view_matrix();

        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = PI / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f * aspect_ratio, 0.0,              0.0              , 0.0],
                [       0.0      ,  f ,              0.0              , 0.0],
                [       0.0      , 0.0,  (zfar+znear)/(zfar-znear)    , 1.0],
                [       0.0      , 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
            ]
        };

        let light: [f32; 3] = [-1.0, 0.4, 0.9];

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        target.draw((&positions, &normals), &indices, &program,
                    &uniform! { model: model, view: view, perspective: perspective, u_light: light },
                    &params).unwrap();
        target.finish().unwrap();

        let (width, height) = window.get_inner_size_points().unwrap();
        let (width, height) = (width as i32 / 2, height as i32 / 2);
        for ev in display.poll_events() {
            use glium::glutin::Event::{ Closed, KeyboardInput, MouseMoved };
            use glium::glutin::ElementState::Pressed;
            use glium::glutin::VirtualKeyCode::*;
            match ev {
                Closed => return,
                KeyboardInput(Pressed, _, Some(Escape)) => return,
                KeyboardInput(state, _, Some(key)) => {
                    let t = state == Pressed;
                    match key {
                        W      => camera.moving_forward  = t,
                        A      => camera.moving_left     = t,
                        S      => camera.moving_backward = t,
                        D      => camera.moving_right    = t,
                        Up     => camera.turning_up      = t,
                        Left   => camera.turning_left    = t,
                        Down   => camera.turning_down    = t,
                        Right  => camera.turning_right   = t,
                        LShift => camera.moving_down     = t,
                        Space  => camera.moving_up       = t,
                        _      => (),
                    }
                },
                MouseMoved((w, h)) => {
                    //screen coordinates increase to the right, just like phi
                    camera.phi += (w - width as i32) as f32 * 0.005;
                    //screen coordinates decrease to the top, unlike theta
                    camera.theta -= (h - height as i32) as f32 * 0.005;
                    camera.norm_phi();
                    camera.norm_theta();
                },
                _ => (),
            }
        }

        window.set_cursor_position(width, height).unwrap();
        camera.update();
    }
}
