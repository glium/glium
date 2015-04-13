pub struct CameraState {
    aspect_ratio: f32,
}

impl CameraState {
    pub fn new() -> CameraState {
        CameraState {
            aspect_ratio: 1024.0 / 768.0,
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
        // note: remember that this is column-major, so the lines of code are actually columns
        [
            [1.0,    0.0,              0.0              ,   0.0],
            [         0.0         ,     1.0 ,              0.0              ,   0.0],
            [         0.0         ,    0.0,  1.0    ,  0.0],
            [         0.0         ,    0.0,  -1.0,   1.0],
        ]
    }
}
