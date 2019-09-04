extern crate glium;
extern crate libc;

use std::ptr;
use std::rc::Rc;

/// Builds a context with dummy OpenGL functions.
pub fn build_context() -> Rc<glium::backend::Context> {
    unsafe {
        glium::backend::Context::new::<_>(DummyBackend, false, Default::default()).unwrap()
    }
}

struct DummyBackend;

unsafe impl glium::backend::Backend for DummyBackend {
    fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const libc::c_void {
        match symbol {
            "glAttachShader" => {
                extern "system" fn attach(_: u32, _: u32) {}
                attach as *const _
            },

            "glBindBuffer" | "glBindTexture" | "glBindSampler" | "glBindFramebuffer" |
            "glBindVertexArray" => {
                extern "system" fn bind(_: u32, _: u32) {}
                bind as *const _
            },

            "glBufferData" => {
                extern "system" fn buffer_data(_: u32, _: isize, _: *const (), _: u32) {}
                buffer_data as *const _
            },

            "glCompileShader" => {
                extern "system" fn compile(_: u32) {}
                compile as *const _
            },

            "glClearColor" => {
                extern "system" fn clear_color(_: f32, _: f32, _: f32, _: f32) {}       // TOD
                clear_color as *const _
            },

            "glClear" => {
                extern "system" fn clear(_: u32) {}
                clear as *const _
            },

            "glCreateProgram" => {
                extern "system" fn create() -> u32 { 1 }
                create as *const _
            },

            "glCreateShader" => {
                extern "system" fn create_sh(_: u32) -> u32 { 1 }
                create_sh as *const _
            },

            "glDeleteFramebuffers" | "glDeleteBuffers" | "glDeleteSamplers" |
            "glDeleteTextures" | "glDeleteVertexArrays" => {
                extern "system" fn delete_mult(_: isize, _: *const u32) {}
                delete_mult as *const _
            },

            "glDeleteProgram" | "glDeleteShader" => {
                extern "system" fn delete(_: u32) {}
                delete as *const _
            },

            "glEnable" | "glDisable" => {
                extern "system" fn enable(_: u32) {}
                enable as *const _
            },

            "glFinish" => {
                extern "system" fn finish() {}
                finish as *const _
            },

            "glGenBuffers" | "glGenTextures" | "glGenFramebuffers" | "glGenRenderbuffers" |
            "glGenVertexArrays" | "glGenSamplers" => {
                extern "system" fn gen(num: usize, bufs: *mut u32) {
                    for i in 0 .. num { unsafe { *bufs.offset(i as isize) = 1; } }
                }
                gen as *const _
            },

            "glGetBooleanv" => {
                extern "system" fn get_booleanv(name: u32, out: *mut u8) {
                    match name {
                        _ => unsafe { *out = 0; },
                    }
                }
                get_booleanv as *const _
            },

            "glGetBufferParameteriv" => {
                extern "system" fn get_buf_paramiv(_: u32, param: u32, out: *mut i32) {
                    match param {
                        0x8764 /* GL_BUFFER_SIZE */ => unsafe { *out = 60 },      // FIXME:
                        _ => unsafe { *out = 0; }
                    }
                }
                get_buf_paramiv as *const _
            }

            "glGetError" => {
                extern "system" fn get_error() -> u32 { 0 }
                get_error as *const _
            },

            "glGetFramebufferAttachmentParameteriv" => {
                extern "system" fn get_fbap(_target: u32, _atch: u32, _pname: u32, _params: *mut i32) {
                }
                get_fbap as *const _
            },

            "glGetIntegerv" => {
                extern "system" fn get_integerv(name: u32, out: *mut i32) {
                    match name {
                        0x821D /* GL_NUM_EXTENSIONS */ => unsafe { *out = 0; },
                        _ => unsafe { *out = 0; },
                    }
                }
                get_integerv as *const _
            },

            "glGetProgramiv" => {
                extern "system" fn get_progiv(_: u32, param: u32, out: *mut i32) {
                    match param {
                        0x8B82 /* GL_LINK_STATUS */ => unsafe { *out = 1; },
                        0x8918 /* GL_GEOMETRY_OUTPUT_TYPE */ => unsafe { *out = 4; /* GL_TRIANGLES */ },
                        _ => unsafe { *out = 0; }
                    }
                }
                get_progiv as *const _
            },

            "glGetShaderiv" => {
                extern "system" fn get_shaderiv(_: u32, param: u32, out: *mut i32) {
                    match param {
                        0x8B81 /* GL_COMPILE_STATUS */ => unsafe { *out = 1 },
                        _ => unsafe { *out = 0; }
                    }
                }
                get_shaderiv as *const _
            },

            "glGetString" => {
                extern "system" fn get_string(name: u32) -> *const i8 {
                    match name {
                        0x1F02 /* GL_VERSION */ => b"3.3 3.3.0\0".as_ptr() as *const _,
                        _ => b"\0".as_ptr() as *const _,
                    }
                }
                get_string as *const _
            },

            "glLinkProgram" => {
                extern "system" fn link(_: u32) {}
                link as *const _
            },

            "glShaderSource" => {
                extern "system" fn shader_source(_: u32, _: isize,
                                                 _: *const *const i8, _: *const i32) {}
                shader_source as *const _
            }

            "glUseProgram" => {
                extern "system" fn use_program(_: u32) {}
                use_program as *const _
            },

            _name => ptr::null()
        }
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (800, 600)
    }

    fn is_current(&self) -> bool {
        true
    }

    unsafe fn make_current(&self) {
    }
}
