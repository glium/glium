/*!

Usage:

```no_run
# extern crate simple_gl;
# extern crate simple_gl_sprite2d;
# fn main() {
# let display: simple_gl::Display = unsafe { std::mem::uninitialized() };
# }
```

*/

#![feature(phase)]
#![deny(missing_doc)]
//#![deny(warnings)]    // TODO: put back

extern crate serialize;
extern crate simple_gl;

use std::sync::Mutex;

mod json;

/// glTF model loaded in OpenGL.
pub struct Model<'d> {
    vertex_buffer: simple_gl::VertexBuffer<u8>,
    index_buffer: simple_gl::IndexBuffer,
    program: Mutex<simple_gl::ProgramUniforms>,
}

/// Object that needs to be passed to allow the library to load misc resources.
pub type ResourcesLoader<'a> = |&str|:'a -> Option<Box<Reader + 'static>>;

impl<'d> Model<'d> {
    /// Loads a glTF model.
    pub fn new<R: Reader>(display: &'d simple_gl::Display, file: R, others_loader: ResourcesLoader)
        -> Model<'d>
    {
        unimplemented!()
    }
}

/// Loads the program with the given name from the document.
fn load_program(display: &simple_gl::Display, name: &str, data: json::GLTFDocument,
                loader: ResourcesLoader) -> Result<Option<simple_gl::Program>, String>
{
    let program = match data.programs.as_ref().and_then(|p| p.find(&name.to_string())) {
        Some(p) => p,
        None => return Ok(None)
    };

    let vertex_shader = match data.shaders.as_ref()
        .and_then(|s| s.find(&program.vertexShader))
    {
        Some(s) => s,
        None => return Err(format!("Couldn't find shader named {} in shaders list",
            program.vertexShader))
    };

    let mut vertex_shader = match loader(vertex_shader.uri.as_slice()) {
        Some(s) => s,
        None => return Err(format!("Couldn't load URI {}", vertex_shader))
    };

    let fragment_shader = match data.shaders.as_ref()
        .and_then(|s| s.find(&program.fragmentShader))
    {
        Some(s) => s,
        None => return Err(format!("Couldn't find shader named {} in shaders list",
            program.fragmentShader))
    };

    let mut fragment_shader = match loader(fragment_shader.uri.as_slice()) {
        Some(s) => s,
        None => return Err(format!("Couldn't load URI {}", fragment_shader))
    };

    let vertex_shader = try!(vertex_shader.read_to_string()
        .map_err(|e| format!("Couldn't read vertex shader file: {}", e)));
    let fragment_shader = try!(fragment_shader.read_to_string()
        .map_err(|e| format!("Couldn't read fragment shader file: {}", e)));

    let program = try!(simple_gl::Program::new(display, vertex_shader.as_slice(),
        fragment_shader.as_slice(), None)
        .map_err(|e| format!("Error while creating program: {}", e)));

    Ok(Some(program))
}
