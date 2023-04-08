use crate::uniforms::{Uniforms, UniformValue, AsUniformValue};
use std::collections::HashMap;

/// Object that can be used when you don't have any uniforms.
#[derive(Debug, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    #[inline]
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, _: F) {
    }
}

/// Stores uniforms.
#[derive(Copy, Clone)]
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
            name,
            value,
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
            name,
            value,
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

/// Stores Uniforms dynamicly in a HashMap.
#[derive(Clone)]
pub struct DynamicUniforms<'a, 's>{
    map: HashMap<&'s str, UniformValue<'a>>,
}

impl<'a, 's> DynamicUniforms<'a, 's>{
    /// Creates new DynamicUniforms
    pub fn new() -> Self{
        Self{
            map: HashMap::new()
        }
    }

    /// Add a value to the DynamicUniforms
    #[inline]
    pub fn add(&mut self, key: &'s str, value: &'a dyn AsUniformValue){
        self.map.insert(key, value.as_uniform_value());
    }
}

impl Uniforms for DynamicUniforms<'_, '_>{
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut output: F) {
        for (key, value) in self.map.iter(){
            output(key, *value);
        }
    }
}
