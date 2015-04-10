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
            let uniforms = $crate::uniforms::UniformsStorage::new(stringify!($field1), $value1);
            $(
                let uniforms = uniforms.add(stringify!($field), $value);
            )+
            uniforms
        }
    };

    ($($field:ident: $value:expr),*,) => {
        uniform!($($field: $value),*)
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
/// #[derive(Copy, Clone)]
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
                use std::borrow::Cow;

                vec![
                    $(
                        (
                            Cow::Borrowed(stringify!($field_name)),
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
    );

    ($struct_name:ident, $($field_name:ident),+,) => (
        implement_vertex!($struct_name, $($field_name),+);
    );
}

/// Implements the `glium::uniforms::Uniforms` trait for the given type.
///
/// The parameters must be the name of the struct and the names of its fields.
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate glium;
/// # fn main() {
/// #[derive(Copy, Clone)]
/// struct Values<'a> {
///     texture: &'a glium::texture::Texture2d,
///     color: [f32; 4]
/// }
///
/// implement_uniforms!(Values, texture, color);
/// # }
/// ```
///
#[macro_export]
macro_rules! implement_uniforms {
    ($struct_name:ident, $($field_name:ident),+) => (
        impl $crate::uniforms::Uniforms for $struct_name {
            fn visit_values<F: FnMut(&str, &$crate::uniforms::UniformValue)>(self, mut output: F) {
                use $crate::uniforms::IntoUniformValue;

                $(
                    output(stringify!($field_name), &IntoUniformValue::into_uniform_value(self.$field_name));
                )+
            }
        }
    );

    ($struct_name:ident, $($field_name:ident),+,) => (
        implement_uniforms!($struct_name, $($field_name),+);
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn trailing_comma_uniform() {
        let u = uniform!{ a: 5, b: 6, };
    }

    #[test]
    fn trailing_comma_impl_vertex() {
        #[derive(Copy, Clone)]
        struct Foo {
            pos: [f32; 2],
        }

        implement_vertex!(Foo, pos,);
    }

    #[test]
    fn trailing_comma_impl_uniforms() {
        #[derive(Copy, Clone)]
        struct Foo {
            val: f32,
        }

        implement_uniforms!(Foo, val,);
    }
}
