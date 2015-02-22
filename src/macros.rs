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

/// Implements the `glium::vertex::Vertex` trait for the given type.
///
/// The parameters must be the name of the struct and the names of its fields.
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate glium;
/// # fn main() {
/// #[derive(Copy)]
/// struct Vertex {
///     position: [f32; 3],
///     tex_coords: [f32; 2],
/// }
///
/// implement_vertex!(Vertex, position, tex_coords);
/// # }
/// ```
///
#[macro_export]
macro_rules! implement_vertex {
    ($struct_name:ident, $($field_name:ident),+) => (
        impl $crate::vertex::Vertex for $struct_name {
            fn build_bindings() -> $crate::vertex::VertexFormat {
                vec![
                    $(
                        (
                            stringify!($field_name).to_string(),
                            {
                                let dummy: &$struct_name = unsafe { ::std::mem::transmute(0usize) };
                                let dummy_field = &dummy.$field_name;
                                let dummy_field: usize = unsafe { ::std::mem::transmute(dummy_field) };
                                dummy_field
                            },
                            {
                                fn attr_type_of_val<T: $crate::vertex::Attribute>(_: &T)
                                    -> $crate::vertex::AttributeType
                                {
                                    <T as $crate::vertex::Attribute>::get_type()
                                }
                                let dummy: &$struct_name = unsafe { ::std::mem::transmute(0usize) };
                                attr_type_of_val(&dummy.$field_name)
                            },
                        )
                    ),+
                ]
            }
        }
    )
}
