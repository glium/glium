use uniforms::{Uniforms, UniformValue, IntoUniformValue};

/// Object that can be used when you don't have any uniforms.
#[derive(Debug, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, _: F) {
    }
}

/// Stores uniforms.
pub struct UniformsStorage<'n, 'u> {
    values: Vec<(&'n str, UniformValue<'u>)>,
}

impl<'n, 'u> UniformsStorage<'n, 'u> {
    /// Builds a new storage with a value.
    pub fn new<T>(name: &'n str, value: T)
                  -> UniformsStorage<'n, 'u>
                  where T: IntoUniformValue<'u>
    {
        UniformsStorage {
            values: vec![(name, value.into_uniform_value())],
        }
    }
}

impl<'n, 'u> UniformsStorage<'n, 'u> {
    /// Adds a value to the storage.
    pub fn add<T>(mut self, name: &'n str, value: T)
                  -> UniformsStorage<'n, 'u>
                  where T: IntoUniformValue<'u>
    {
        self.values.push((name, value.into_uniform_value()));
        self
    }
}

impl<'a, 'n, 'u> Uniforms for &'a UniformsStorage<'n, 'u> {
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, mut output: F) {
        for &(n, ref v) in &self.values {
            output(n, v)
        }
    }
}
