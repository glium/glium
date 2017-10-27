use std::marker::PhantomData;

use uniforms::{Uniforms, UniformValue};

/// Object that can be used when you don't have any uniforms.
#[derive(Debug, Copy, Clone)]
pub struct EmptyUniforms;

impl Uniforms for EmptyUniforms {
    fn visit_values<F>(&self, _: F)
    where
        for<'a> F: FnMut(&str, &UniformValue<'a>),
    {
    }
}

/// Stores uniforms.
pub struct UniformsStorage<'a, 'n, T: 'a, R>
where
    T: Into<UniformValue<'a>> + Clone,
    R: Uniforms,
{
    name: &'n str,
    value: T,
    rest: R,
    marker: PhantomData<&'a T>,
}

impl<'a, 'n, T: 'a> UniformsStorage<'a, 'n, T, EmptyUniforms>
where
    T: Into<UniformValue<'a>> + Clone,
{
    /// Builds a new storage with a value.
    #[inline]
    pub fn new(name: &'n str, value: T) -> UniformsStorage<'a, 'n, T, EmptyUniforms> {
        UniformsStorage {
            name: name,
            value: value,
            rest: EmptyUniforms,
            marker: PhantomData,
        }
    }
}

impl<'a, 'n, T: 'a, R> UniformsStorage<'a, 'n, T, R>
where
    T: Into<UniformValue<'a>> + Clone,
    R: Uniforms,
{
    /// Adds a value to the storage.
    #[inline]
    pub fn add<U>(
        self,
        name: &'n str,
        value: U,
    ) -> UniformsStorage<'a, 'n, U, UniformsStorage<'a, 'n, T, R>>
    where
        U: Into<UniformValue<'a>> + Clone,
    {
        UniformsStorage {
            name: name,
            value: value,
            rest: self,
            marker: PhantomData,
        }
    }
}

impl<'a, 'n, T: 'a, R> Uniforms for UniformsStorage<'a, 'n, T, R>
where
    T: Into<UniformValue<'a>> + Clone,
    R: Uniforms,
{
    #[inline]
    fn visit_values<F>(&self, mut output: F)
    where
        for<'b> F: FnMut(&str, &UniformValue<'b>),
    {
        output(self.name, &self.value.to_owned().into());
        self.rest.visit_values(output);
    }
}
