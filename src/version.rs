use std::cmp::Ordering;
use std::ffi::CStr;
use gl;

/// Describes a version.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version(pub Api, pub u8, pub u8);

/// Describes the corresponding API.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Api {
    /// Regular OpenGL.
    Gl,
    /// OpenGL embedded system.
    GlEs,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        if self.0 != other.0 {
            return None;
        }

        match self.1.cmp(&other.1) {
            Ordering::Equal => Some(self.2.cmp(&other.2)),
            a => Some(a)
        }
    }
}

pub fn get_gl_version(gl: &gl::Gl) -> Version {
    unsafe {
        let version = gl.GetString(gl::VERSION) as *const i8;
        let version = String::from_utf8(CStr::from_ptr(version).to_bytes().to_vec()).unwrap();

        let (version, gles) = if version.starts_with("OpenGL ES ") {
            (&version[10..], true)
        } else {
            (&version[..], false)
        };

        let version = version.words().next().expect("glGetString(GL_VERSION) returned an empty \
                                                     string");

        let mut iter = version.split(move |c: char| c == '.');
        let major = iter.next().unwrap();
        let minor = iter.next().expect("glGetString(GL_VERSION) did not return a correct version");

        Version(
            if gles { Api::GlEs } else { Api::Gl },
            major.parse().ok().expect("failed to parse GL major version"),
            minor.parse().ok().expect("failed to parse GL minor version"),
        )
    }
}

/// Given an API version, this function returns the GLSL version that the implementation is
/// required to support.
pub fn get_supported_glsl_version(gl_version: &Version) -> Version {
    match gl_version.0 {
        Api::Gl => {
            // since OpenGL 3.3: glsl versions match gl version, just return a copy
            if *gl_version >= Version(gl_version.0, 3, 3) {
                return *gl_version;
            }

            // match to detect invalid versions
            match *gl_version {
                Version(a, 2, 0) => Version(a, 1, 1),
                Version(a, 2, 1) => Version(a, 1, 2),
                Version(a, 3, 0) => Version(a, 1, 3),
                Version(a, 3, 1) => Version(a, 1, 4),
                Version(a, 3, 2) => Version(a, 1, 5),
                _ => panic!("no corresponding glsl version exists")
            }
        },
        Api::GlEs => {
            // since OpenGL ES 3.0: glsl versions match gl version, just return a copy
            if *gl_version >= Version(gl_version.0, 3, 0) {
                return *gl_version;
            }

            // only other valid GLES version is 2.0
            if *gl_version == Version(gl_version.0, 2, 0){
                return Version(Api::GlEs, 1, 0);
            } else {
                panic!("no corresponding glsl version exists")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Version, Api, get_supported_glsl_version};

    macro_rules! assert_versions {
        ( $api:path, $gl_major:expr, $gl_minor:expr => $glsl_major:expr, $glsl_minor:expr) => {
            assert_eq!(
                get_supported_glsl_version(&Version($api, $gl_major, $gl_minor)),
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
    #[should_panic]
    fn invalid_gl_version() {
        get_supported_glsl_version(&Version(Api::Gl, 1, 0));
    }

    #[test]
    #[should_panic]
    fn invalid_gles_version() {
        get_supported_glsl_version(&Version(Api::GlEs, 1, 0));
    }
}
