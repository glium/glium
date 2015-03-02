use std::option::IntoIter;
use std::iter::Chain;
use std::cell::RefCell;

use uniforms::{Uniforms, UniformValue, IntoUniformValue};

/// Object that can be used when you don't have any uniforms.
#[derive(Debug, Copy, Clone)]
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
/// // the final type is very complex, but you don't care about it
/// ```
///
pub struct UniformsStorage<I> {
    // FIXME: this RefCell<Option<>> hack is here because the `Uniforms` trait is implemented
    //        only on &UniformsStorage
    iterator: RefCell<Option<I>>,
}

impl<'n, 'u> UniformsStorage<IntoIter<(&'n str, UniformValue<'u>)>> {
    /// Builds a new storage with a value.
    pub fn new<T>(name: &'n str, value: T)
                  -> UniformsStorage<IntoIter<(&'n str, UniformValue<'u>)>>
                  where T: IntoUniformValue<'u>
    {
        UniformsStorage {
            iterator: RefCell::new(Some(Some((name, value.into_uniform_value())).into_iter())),
        }
    }
}

impl<'n, 'u, I> UniformsStorage<I> where I: Iterator<Item = (&'n str, UniformValue<'u>)> {
    /// Adds a value to the storage.
    pub fn add<T>(self, name: &'n str, value: T)
                  -> UniformsStorage<Chain<I, IntoIter<(&'n str, UniformValue<'u>)>>>
                  where T: IntoUniformValue<'u>
    {
        let iter = self.iterator.borrow_mut().take().unwrap();

        UniformsStorage {
            iterator: RefCell::new(Some(iter.chain(Some((name, value.into_uniform_value())).into_iter())))
        }
    }
}

impl<'a, 'n, 'u, I: 'a> Uniforms for &'a UniformsStorage<I>
                                     where I: Iterator<Item = (&'n str, UniformValue<'u>)>
{
    fn visit_values<F: FnMut(&str, &UniformValue)>(self, mut output: F) {
        let iterator = self.iterator.borrow_mut().take().unwrap();
        for (n, v) in iterator {
            output(n, &v)
        }
    }
}
