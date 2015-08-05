use uniforms::{Uniforms, UniformValue, AsUniformValue};

/// Object that can be used when you don't have any uniforms.
#[derive(Debug, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    #[inline]
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, _: F) {
    }
}

/// Stores uniforms.
pub struct UniformsStorage<'n, T, R> where T: AsUniformValue, R: Uniforms {
    name: &'n str,
    value: T,
    rest: R,
}

impl<'n, T> UniformsStorage<'n, T, EmptyUniforms> where T: AsUniformValue {
    /// Builds a new storage with a value.
    #[inline]
    pub fn new(name: &'n str, value: T)
               -> UniformsStorage<'n, T, EmptyUniforms>
    {
        UniformsStorage {
            name: name,
            value: value,
            rest: EmptyUniforms,
        }
    }
}

impl<'n, T, R> UniformsStorage<'n, T, R> where T: AsUniformValue, R: Uniforms {
    /// Adds a value to the storage.
    #[inline]
    pub fn add<U>(self, name: &'n str, value: U)
                  -> UniformsStorage<'n, U, UniformsStorage<'n, T, R>>
                  where U: AsUniformValue
    {
        UniformsStorage {
            name: name,
            value: value,
            rest: self,
        }
    }
}

impl<'n, T, R> Uniforms for UniformsStorage<'n, T, R> where T: AsUniformValue, R: Uniforms {
    #[inline]
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut output: F) {
        output(self.name, self.value.as_uniform_value());
        self.rest.visit_values(output);
    }
}
