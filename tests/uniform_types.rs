extern crate glium;
use glium::uniforms::{UniformValue};

#[test]
fn uniform_from_i8() {
    let tested_value: i8 = -1;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::SignedInt(tested_value as i32));
}

#[test]
fn uniform_from_i16() {
    let tested_value: i16 = -2550;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::SignedInt(tested_value as i32));
}

#[test]
fn uniform_from_i32() {
    let tested_value: i32 = -2550;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::SignedInt(tested_value as i32));
}

#[test]
fn uniform_from_i32_2_tuple() {
    let tested_value: (i32, i32) = (0, -1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::IntVec2([0, -1]));
}

#[test]
fn uniform_from_i32_3_tuple() {
    let tested_value: (i32, i32, i32) = (0, -1, 1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::IntVec3([0, -1, 1]));
}

#[test]
fn uniform_from_i32_4_tuple() {
    let tested_value: (i32, i32, i32, i32) = (0, -1, 1, -1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::IntVec4([0, -1, 1, -1]));
}

#[test]
fn uniform_from_i32_2_array() {
    let tested_value: [i32; 2] = [0, -1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::IntVec2(tested_value));
}

#[test]
fn uniform_from_i32_3_array() {
    let tested_value: [i32; 3] = [0, -1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::IntVec3(tested_value));
}

#[test]
fn uniform_from_i32_4_array() {
    let tested_value: [i32; 4] = [0, -1, 1, -1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::IntVec4(tested_value));
}

#[test]
fn uniform_from_u8() {
    let tested_value: u8 = 255;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt(tested_value as u32));
}

#[test]
fn uniform_from_u16() {
    let tested_value: u16 = 2550;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt(tested_value as u32));
}

#[test]
fn uniform_from_u32() {
    let tested_value: u32 = 2550;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt(tested_value as u32));
}

#[test]
fn uniform_from_u32_2_array() {
    let tested_value: [u32; 2] = [0, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedIntVec2(tested_value));
}

#[test]
fn uniform_from_u32_3_array() {
    let tested_value: [u32; 3] = [0, 1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedIntVec3(tested_value));
}

#[test]
fn uniform_from_u32_4_array() {
    let tested_value: [u32; 4] = [0, 2, 1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedIntVec4(tested_value));
}

#[test]
fn uniform_from_u32_2_tuple() {
    let tested_value: (u32, u32) = (0, 1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedIntVec2([0, 1]));
}

#[test]
fn uniform_from_u32_3_tuple() {
    let tested_value: (u32, u32, u32) = (0, 1, 2);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedIntVec3([0, 1, 2]));
}

#[test]
fn uniform_from_u32_4_tuple() {
    let tested_value: (u32, u32, u32, u32) = (0, 1, 2, 3);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedIntVec4([0, 1, 2, 3]));
}

#[test]
fn uniform_from_bool_2_array() {
    let tested_value: [bool; 2] = [true, false];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::BoolVec2(tested_value));
}

#[test]
fn uniform_from_bool_3_array() {
    let tested_value: [bool; 3] = [true, false, true];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::BoolVec3(tested_value));
}

#[test]
fn uniform_from_bool_4_array() {
    let tested_value: [bool; 4] = [true, false, false, true];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::BoolVec4(tested_value));
}

#[test]
fn uniform_from_bool() {
    let tested_value: bool = true;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Bool(true));
}

#[test]
fn uniform_from_bool_2_tuple() {
    let tested_value: (bool, bool) = (true, false);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::BoolVec2([true, false]));
}

#[test]
fn uniform_from_bool_3_tuple() {
    let tested_value: (bool, bool, bool) = (true, false, false);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::BoolVec3([true, false, false]));
}

#[test]
fn uniform_from_bool_4_tuple() {
    let tested_value: (bool, bool, bool, bool) = (true, false, true, false);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::BoolVec4([true, false, true, false]));
}

#[test]
fn uniform_from_f32() {
    let tested_value: f32 = 1.;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Float(tested_value as f32));
}

#[test]
fn uniform_from_f32_2_array() {
    let tested_value: [f32; 2] = [0., 1.];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Vec2(tested_value));
}

#[test]
fn uniform_from_f32_3_array() {
    let tested_value: [f32; 3] = [0., 1., 1.];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Vec3(tested_value));
}

#[test]
fn uniform_from_f32_4_array() {
    let tested_value: [f32; 4] = [0., 2., 1., 1.];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Vec4(tested_value));
}

#[test]
fn uniform_from_f32_2_tuple() {
    let tested_value: (f32, f32) = (0., -1.);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Vec2([0., -1.]));
}

#[test]
fn uniform_from_f32_3_tuple() {
    let tested_value: (f32, f32, f32) = (0., -1., 1.);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Vec3([0., -1., 1.]));
}

#[test]
fn uniform_from_f32_4_tuple() {
    let tested_value: (f32, f32, f32, f32) = (0., -1., 1., -1.);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Vec4([0., -1., 1., -1.]));
}

#[test]
fn uniform_from_f64() {
    let tested_value: f64 = 1.;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Double(tested_value as f64));
}

#[test]
fn uniform_from_f64_2_array() {
    let tested_value: [f64; 2] = [0., 1.];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleVec2(tested_value));
}

#[test]
fn uniform_from_f64_3_array() {
    let tested_value: [f64; 3] = [0., 1., 1.];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleVec3(tested_value));
}

#[test]
fn uniform_from_f64_4_array() {
    let tested_value: [f64; 4] = [0., 2., 1., 1.];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleVec4(tested_value));
}

#[test]
fn uniform_from_f64_2_tuple() {
    let tested_value: (f64, f64) = (0., -1.);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleVec2([0., -1.]));
}

#[test]
fn uniform_from_f64_3_tuple() {
    let tested_value: (f64, f64, f64) = (0., -1., 1.);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleVec3([0., -1., 1.]));
}

#[test]
fn uniform_from_f64_4_tuple() {
    let tested_value: (f64, f64, f64, f64) = (0., -1., 1., -1.);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleVec4([0., -1., 1., -1.]));
}

#[test]
fn uniform_from_i64() {
    let tested_value: i64 = 1;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64(tested_value as i64));
}

#[test]
fn uniform_from_i64_2_array() {
    let tested_value: [i64; 2] = [0, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64Vec2(tested_value));
}

#[test]
fn uniform_from_i64_3_array() {
    let tested_value: [i64; 3] = [0, 1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64Vec3(tested_value));
}

#[test]
fn uniform_from_i64_4_array() {
    let tested_value: [i64; 4] = [0, 2, 1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64Vec4(tested_value));
}

#[test]
fn uniform_from_i64_2_tuple() {
    let tested_value: (i64, i64) = (0, -1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64Vec2([0, -1]));
}

#[test]
fn uniform_from_i64_3_tuple() {
    let tested_value: (i64, i64, i64) = (0, -1, 1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64Vec3([0, -1, 1]));
}

#[test]
fn uniform_from_i64_4_tuple() {
    let tested_value: (i64, i64, i64, i64) = (0, -1, 1, -1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Int64Vec4([0, -1, 1, -1]));
}

#[test]
fn uniform_from_u64() {
    let tested_value: u64 = 1;
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64(tested_value as u64));
}

#[test]
fn uniform_from_u64_2_array() {
    let tested_value: [u64; 2] = [0, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64Vec2(tested_value));
}

#[test]
fn uniform_from_u64_3_array() {
    let tested_value: [u64; 3] = [0, 1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64Vec3(tested_value));
}

#[test]
fn uniform_from_u64_4_array() {
    let tested_value: [u64; 4] = [0, 2, 1, 1];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64Vec4(tested_value));
}

#[test]
fn uniform_from_u64_2_tuple() {
    let tested_value: (u64, u64) = (0, 1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64Vec2([0, 1]));
}

#[test]
fn uniform_from_u64_3_tuple() {
    let tested_value: (u64, u64, u64) = (0, 1, 1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64Vec3([0, 1, 1]));
}

#[test]
fn uniform_from_u64_4_tuple() {
    let tested_value: (u64, u64, u64, u64) = (0, 1, 1, 1);
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::UnsignedInt64Vec4([0, 1, 1, 1]));
}

#[test]
fn unfirom_from_f32_2x2_matrix() {
    let tested_value: [[f32; 2]; 2] = [[1., 0.], [0., 1.]];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::Mat2([[1., 0.], [0., 1.]]));
}

#[test]
fn unfirom_from_f32_3x3_matrix() {
    let tested_value: [[f32; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let result: UniformValue = tested_value.into();
    assert_eq!(
        result,
        UniformValue::Mat3([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
    );
}

#[test]
fn unfirom_from_f32_4x4_matrix() {
    let tested_value: [[f32; 4]; 4] = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [1., 0., 0., 1.],
    ];
    let result: UniformValue = tested_value.into();
    assert_eq!(
        result,
        UniformValue::Mat4(
            [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [1., 0., 0., 1.],
            ],
        )
    );
}

#[test]
fn unfirom_from_f64_2x2_matrix() {
    let tested_value: [[f64; 2]; 2] = [[1., 0.], [0., 1.]];
    let result: UniformValue = tested_value.into();
    assert_eq!(result, UniformValue::DoubleMat2([[1., 0.], [0., 1.]]));
}

#[test]
fn unfirom_from_f64_3x3_matrix() {
    let tested_value: [[f64; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let result: UniformValue = tested_value.into();
    assert_eq!(
        result,
        UniformValue::DoubleMat3([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
    );
}

#[test]
fn unfirom_from_f64_4x4_matrix() {
    let tested_value: [[f64; 4]; 4] = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [1., 0., 0., 1.],
    ];
    let result: UniformValue = tested_value.into();
    assert_eq!(
        result,
        UniformValue::DoubleMat4(
            [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [1., 0., 0., 1.],
            ],
        )
    );
}
