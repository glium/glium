use std::cmp::Ordering;
use std::ffi;
use gl;

/// Describes an OpenGL ctxt.version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlVersion(pub u8, pub u8);

impl PartialOrd for GlVersion {
    fn partial_cmp(&self, other: &GlVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GlVersion {
    fn cmp(&self, other: &GlVersion) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => self.1.cmp(&other.1),
            a => a
        }
    }
}

pub fn get_gl_version(gl: &gl::Gl) -> GlVersion {
    unsafe {
        let version = gl.GetString(gl::VERSION) as *const i8;
        let version = String::from_utf8(ffi::c_str_to_bytes(&version).to_vec()).unwrap();

        let version = version.words().next().expect("glGetString(GL_VERSION) returned an empty \
                                                     string");

        let mut iter = version.split(move |c: char| c == '.');
        let major = iter.next().unwrap();
        let minor = iter.next().expect("glGetString(GL_VERSION) did not return a correct version");

        GlVersion(
            major.parse().ok().expect("failed to parse GL major version"),
            minor.parse().ok().expect("failed to parse GL minor version"),
        )
    }
}
