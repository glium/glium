use uniforms::{Uniforms, UniformValue, IntoUniformValue};

/// Object that can be used when you don't have any uniform.
#[deriving(Show, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, _: F) {
    }
}

/// Stores uniforms.
///
/// # Example
///
/// ```ignore   // TODO: CRASHES RUSTDOC OTHERWISE
/// use glium::uniforms::UniformsStorage;
///
/// // `name1` will contain 2.0
/// let uniforms = UniformsStorage::new("name1", 2.0f32);
///
/// // `name2` will contain -0.5
/// let uniforms = uniforms.add("name2", -0.5f32);
///
/// // `name3` will contain `texture`
/// # let texture: glium::Texture2d = unsafe { ::std::mem::uninitialized() };
/// let uniforms = uniforms.add("name3", &texture);
///
/// // the final type is `UniformsStorage<&Texture2d, UniformsStorage<f32, UniformsStorage<f32, EmptyUniforms>>>`
/// // but you shouldn't care about it
/// ```
///
pub struct UniformsStorage<'a> {
    uniforms: Vec<(&'a str, UniformValue<'a>)>,
}

impl<'a> UniformsStorage<'a> {
    /// Builds a new storage with a value.
    pub fn new<T>(name: &'a str, value: T) -> UniformsStorage<'a> where T: IntoUniformValue<'a> {
        UniformsStorage {
            uniforms: vec![(name, value.into_uniform_value())]
        }
    }

    /// Adds a value to the storage.
    pub fn add<T>(mut self, name: &'a str, value: T) -> UniformsStorage<'a>
                  where T: IntoUniformValue<'a>
    {
        self.uniforms.push((name, value.into_uniform_value()));
        self
    }
}

impl<'a: 'b, 'b> Uniforms for &'b UniformsStorage<'a> {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, mut output: F) {
        for &(n, ref v) in self.uniforms.iter() {
            output(n, v)
        }
    }
}
