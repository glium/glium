extern crate glium;

use glium::{Version, Api};

macro_rules! assert_versions {
    ( $api:path, $gl_major:expr, $gl_minor:expr => $glsl_major:expr, $glsl_minor:expr) => {
        assert_eq!(
            Version($api, $gl_major, $gl_minor).get_glsl_version(),
            Version($api, $glsl_major, $glsl_minor)
                );
    }
}

#[test]
fn valid_gl_versions() {
    // irregular versions
    assert_versions!(Api::Gl, 2, 0 => 1, 1);
    assert_versions!(Api::Gl, 2, 1 => 1, 2);
    assert_versions!(Api::Gl, 3, 0 => 1, 3);
    assert_versions!(Api::Gl, 3, 1 => 1, 4);
    assert_versions!(Api::Gl, 3, 2 => 1, 5);

    // test a few regular versions
    assert_versions!(Api::Gl, 3, 3 => 3, 3);
    assert_versions!(Api::Gl, 4, 0 => 4, 0);
    assert_versions!(Api::Gl, 4, 5 => 4, 5);
}

#[test]
fn valid_gles_versions() {
    // only irregular version
    assert_versions!(Api::GlEs, 2, 0 => 1, 0);

    // some regular versions
    assert_versions!(Api::GlEs, 3, 0 => 3, 0);
    assert_versions!(Api::GlEs, 3, 1 => 3, 1);
}

#[test]
#[should_fail]
fn invalid_gl_version() {
   Version(Api::Gl, 1, 0).get_glsl_version();
}

#[test]
#[should_fail]
fn invalid_gles_version() {
   Version(Api::GlEs, 1, 0).get_glsl_version();
}
