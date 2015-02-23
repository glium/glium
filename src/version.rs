use std::cmp::Ordering;
use std::ffi;
use gl;

/// Describes a version.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version(pub Api, pub u8, pub u8);

/// Describes the corresponding API.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Api {
    Gl,
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
        let version = String::from_utf8(ffi::c_str_to_bytes(&version).to_vec()).unwrap();

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
