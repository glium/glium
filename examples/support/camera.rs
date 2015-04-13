pub struct CameraState {
    aspect_ratio: f32,
    position: (f32, f32, f32),
    target: (f32, f32, f32),
}

impl CameraState {
    pub fn new() -> CameraState {
        CameraState {
            aspect_ratio: 1024.0 / 768.0,
            position: (0.1, 0.1, 1.0),
            target: (0.0, 0.0, 0.0),
        }
    }

    pub fn get_perspective(&self) -> [[f32; 4]; 4] {
        let fov: f32 = 3.141592 / 2.0;
        let zfar = 25.0;
        let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        // note: remember that this is column-major, so the lines of code are actually columns
        [
            [f / self.aspect_ratio,    0.0,              0.0              ,   0.0],
            [         0.0         ,     f ,              0.0              ,   0.0],
            [         0.0         ,    0.0,  (zfar+znear)/(znear-zfar)    ,  -1.0],
            [         0.0         ,    0.0,  (2.0*zfar*znear)/(znear-zfar),   0.0],
        ]
    }

    pub fn get_view(&self) -> [[f32; 4]; 4] {
        let f = (self.target.0 - self.position.0,
                 self.target.1 - self.position.1,
                 self.target.2 - self.position.2);

        let f = {
            let len = f.0 * f.0 + f.1 * f.1 + f.2 * f.2;
            let len = len.sqrt();
            (f.0 / len, f.1 / len, f.2 / len)
        };

        let up = (0.0, 1.0, 0.0);

        let s = (f.1 * up.2 - f.2 * up.1,
                 f.2 * up.0 - f.0 * up.2,
                 f.0 * up.1 - f.1 * up.0);

        let s_norm = {
            let len = s.0 * s.0 + s.1 * s.1 + s.2 * s.2;
            let len = len.sqrt();
            (s.0 / len, s.1 / len, s.2 / len)
        };

        let u = (s_norm.1 * f.2 - s_norm.2 * f.1,
                 s_norm.2 * f.0 - s_norm.0 * f.2,
                 s_norm.0 * f.1 - s_norm.1 * f.0);

        // note: remember that this is column-major, so the lines of code are actually columns
        [
            [s.0, u.0, -f.0, 0.0],
            [s.1, u.1, -f.1, 0.0],
            [s.2, u.2, -f.2, 0.0],
            [-self.position.0, -self.position.1, -self.position.2, 1.0],
        ]
    }
}
