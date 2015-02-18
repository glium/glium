#![cfg(feature = "nalgebra")]

extern crate nalgebra;

use self::nalgebra::Persp3;

pub struct CameraState;

impl CameraState {
    pub fn new() -> CameraState {
        CameraState
    }

    pub fn get_perspective(&self) -> nalgebra::Mat4<f32> {
        Persp3::new(0.75, 45.0, 0.1, 25.0).to_mat()
    }
}
