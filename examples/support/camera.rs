use glium::{ self, glutin };

use std::f32::consts::PI;

const CAM_MOVE_STEP: f32 = 0.02;
const CAM_TURN_STEP: f32 = 0.02;
const CAM_TURN_STEP_MOUSE: f32 = 0.005;

const UP: [f32; 3] = [0.0, 1.0, 0.0];

pub struct CameraState {
    position: [f32; 3],
    // horizontal angle relative to -z, increasing to the right
    // also known as yaw or azimuth angle
    phi: f32,
    // vertical angle, relative to the xz-plane, increasing to the top
    // also known as pitch or altitude angle
    theta: f32,

    moving_up: bool,
    moving_left: bool,
    moving_down: bool,
    moving_right: bool,
    moving_forward: bool,
    moving_backward: bool,
    turning_up: bool,
    turning_left: bool,
    turning_down: bool,
    turning_right: bool,
}

impl CameraState {
    pub fn new() -> CameraState {
        CameraState::with_angles([0.0, 0.0, 1.0], 0.0, 0.0)
    }

    pub fn with_angles(pos: [f32; 3], phi: f32, theta: f32) -> CameraState {
        let mut cam = CameraState {
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

    pub fn at(pos: [f32; 3], target: [f32; 3]) -> CameraState {
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

        CameraState::with_angles(pos, phi, theta)
    }

    fn norm_phi(&mut self) {
        while self.phi < 0.0 {
            self.phi += PI * 2.0;
        }
        self.phi %= PI * 2.0;
    }

    fn norm_theta(&mut self) {
        self.theta = self.theta.max(-PI * 0.4999).min(PI * 0.4999);
    }

    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
    }

    pub fn get_perspective(&self, frame_dimensions: (u32, u32)) -> [[f32; 4]; 4] {
        let (width, height) = frame_dimensions;
        let aspect_ratio = height as f32 / width as f32;
        let fov: f32 = 3.141592 / 3.0;
        let zfar = 1024.0;
        let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        // note: remember that this is column-major, so the lines of code are actually columns
            [
                [f * aspect_ratio, 0.0,              0.0              , 0.0],
                [       0.0      ,  f ,              0.0              , 0.0],
                [       0.0      , 0.0,  (zfar+znear)/(zfar-znear)    , 1.0],
                [       0.0      , 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
            ]
    }

    pub fn direction(&self) -> [f32; 3] {
        [
            self.theta.cos() * self.phi.sin(),
            self.theta.sin(),
            -self.theta.cos() * self.phi.cos(),
        ]
    }

    pub fn get_view(&self) -> [[f32; 4]; 4] {
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

    pub fn update(&mut self, display: &glium::Display) {
        let window = display.get_window().unwrap();
        let (width, height) = window.get_inner_size_points().unwrap();
        let (width, height) = (width as i32 / 2, height as i32 / 2);
        window.set_cursor_position(width, height).unwrap();

        if self.moving_up {
            self.position[1] += CAM_MOVE_STEP;
        }

        if self.moving_left {
            let a = self.phi + PI * 1.5;
            self.position[0] += a.sin() * CAM_MOVE_STEP;
            self.position[2] -= a.cos() * CAM_MOVE_STEP;
        }

        if self.moving_down {
            self.position[1] -= CAM_MOVE_STEP;
        }

        if self.moving_right {
            let a = self.phi + PI * 0.5;
            self.position[0] += a.sin() * CAM_MOVE_STEP;
            self.position[2] -= a.cos() * CAM_MOVE_STEP;
        }

        if self.moving_forward {
            let a = self.phi + PI * 0.0;
            self.position[0] += a.sin() * CAM_MOVE_STEP;
            self.position[2] -= a.cos() * CAM_MOVE_STEP;
        }

        if self.moving_backward {
            let a = self.phi + PI * 1.0;
            self.position[0] += a.sin() * CAM_MOVE_STEP;
            self.position[2] -= a.cos() * CAM_MOVE_STEP;
        }
        if self.turning_up {
            self.theta += CAM_TURN_STEP
        }
        if self.turning_left {
            self.phi -= CAM_TURN_STEP
        }
        if self.turning_down {
            self.theta -= CAM_TURN_STEP
        }
        if self.turning_right {
            self.phi += CAM_TURN_STEP
        }
        self.norm_phi();
        self.norm_theta();
    }

    pub fn process_input(&mut self, event: &glutin::Event, frame_dimensions: (u32, u32)) {
        use glium::glutin::Event::{ KeyboardInput, MouseMoved };
        use glium::glutin::ElementState::Pressed;
        use glium::glutin::VirtualKeyCode::*;
        match *event {
            KeyboardInput(state, _, Some(key)) => {
                let t = state == Pressed;
                match key {
                    W      => self.moving_forward  = t,
                    A      => self.moving_left     = t,
                    S      => self.moving_backward = t,
                    D      => self.moving_right    = t,
                    Up     => self.turning_up      = t,
                    Left   => self.turning_left    = t,
                    Down   => self.turning_down    = t,
                    Right  => self.turning_right   = t,
                    LShift => self.moving_down     = t,
                    Space  => self.moving_up       = t,
                    _      => (),
                }
            },
            MouseMoved((w, h)) => {
                let (width, height) = (frame_dimensions.0 as i32 / 2, frame_dimensions.1 as i32 / 2);
                //screen coordinates increase to the right, just like phi
                self.phi += (w - width) as f32 * CAM_TURN_STEP_MOUSE;
                //screen coordinates decrease to the top, unlike theta
                self.theta -= (h - height) as f32 * CAM_TURN_STEP_MOUSE;
                self.norm_phi();
                self.norm_theta();
            },
            _ => (),
        }
    }
}
