//! Defines useful macros for glium usage.

/// Returns an implementation-defined type which implements the `Uniform` trait.
///
/// ## Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate glium;
/// # fn main() {
/// let uniforms = uniform! {
///     color: [1.0, 1.0, 0.0, 1.0],
///     some_value: 12i32
/// };
/// # }
/// ```
#[macro_export]
macro_rules! uniform {
    () => {
        $crate::uniforms::EmptyUniforms
    };

    ($field:ident: $value:expr) => {
        $crate::uniforms::UniformsStorage::new(stringify!($field), $value)
    };

    ($field1:ident: $value1:expr, $($field:ident: $value:expr),+) => {
        {
            let mut uniforms = $crate::uniforms::UniformsStorage::new(stringify!($field1), $value1);
            $(
                uniforms = uniforms.add(stringify!($field), $value);
            )+
            uniforms
        }
    };
}

/*
// TODO: doesn't work
#[macro_export]
macro_rules! attributes {
    ($(#[$attr:meta])* struct $struct_name:ident {
        $($(#[$field_attr:meta])* $field:ident: $t:ty),*
    }) => {
        #[derive(Copy)]
        $(#[$attr])*
        pub struct $struct_name {
            $(
                $($field_attr)* pub $field: $t
            ),+
        }

        impl $crate::vertex::Vertex for $struct_name {
            fn build_bindings() -> $crate::vertex::VertexFormat {
                vec![
                    $(
                        (
                            stringify!($field),
                            {
                                let dummy: &$struct_name = unsafe { ::std::mem::transmute(0u) };
                                let dummy_field = &dummy.$field;
                                let dummy_field: usize = unsafe { ::std::mem::transmute(dummy_field) };
                                dummy_field
                            },
                            $crate::vertex::Attribute::get_type(None::<$t>)
                        )
                    )+
                ]
            }
        }
    }
}*/
