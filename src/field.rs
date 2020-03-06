//! Field utils

use std::marker::PhantomData;

/// Represents the field of a struct
///
/// Created with the `field!` macro.
pub struct Field<T> {
    offs: usize,
    size: usize,
    phantom: PhantomData<T>,
}

impl<T> Field<T> {
    /// Gets the offset of the field
    pub const fn offs(&self) -> usize {
        self.offs
    }
    /// Gets the size of the field
    pub const fn size(&self) -> usize {
        self.size
    }
}

#[doc(hidden)]
pub fn _hidden_field<T>(offs: usize, _: Option<&T>) -> Field<T> {
    Field {
        offs,
        size: std::mem::size_of::<T>(),
        phantom: PhantomData,
    }
}

/// A macro to create a `Field`.
#[macro_export]
macro_rules! field {
    ($struct_name:ident, $field_name:ident) => {{
        let opt = None::<&$struct_name>.map(|v| &v.$field_name);
        let offs = $crate::__glium_offset_of!($struct_name, $field_name);
        $crate::field::_hidden_field(offs, opt)
    }};
}
