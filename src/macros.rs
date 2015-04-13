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

/// Builds a program depending on the GLSL version supported by the backend.
///
/// This is implemented with successive calls to `is_glsl_version_supported()`.
///
/// ## Example
///
/// ```ignore       // TODO: no_run instead
/// # #[macro_use]
/// # extern crate glium;
/// # fn main() {
/// # let display: glium::Display = unsafe { std::mem::uninitialized() };
/// let program = program!(&display,
///     300 => {
///         vertex: r#"
///             #version 300
///             
///             fn main() {
///                 gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
///             }
///         "#,
///         fragment: r#"
///             #version 300
///
///             out vec4 color;
///             fn main() {
///                 color = vec4(1.0, 1.0, 0.0, 1.0);
///             }
///         "#,
///     },
///     110 => {
///         vertex: r#"
///             #version 110
///             
///             fn main() {
///                 gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
///             }
///         "#,
///         fragment: r#"
///             #version 110
///
///             fn main() {
///                 gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0);
///             }
///         "#,
///     },
///     300 es => {
///         vertex: r#"
///             #version 110
///             
///             fn main() {
///                 gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
///             }
///         "#,
///         fragment: r#"
///             #version 110
///
///             fn main() {
///                 gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0);
///             }
///         "#,
///     },
/// );
/// # }
/// ```
///
#[macro_export]
macro_rules! program {
    ($facade:expr,) => (
        panic!("No version found");     // TODO: handle better
    );
    
    ($facade:expr,,$($rest:tt)*) => (
        program!($facade,$($rest)*)
    );

    ($facade:expr, $num:tt => $($rest:tt)*) => (
        {
            let context = $crate::backend::Facade::get_context($facade);
            let version = program!(_parse_num_gl $num);
            program!(_inner, context, version, $($rest)*)
        }
    );

    ($facade:expr, $num:tt es => $($rest:tt)*) => (
        {
            let context = $crate::backend::Facade::get_context($facade);
            let version = program!(_parse_num_gles $num);
            program!(_inner, context, version, $($rest)*)
        }
    );

    (_inner, $context:ident, $vers:ident, {$($ty:ident:$src:expr),+}$($rest:tt)*) => (
        if $context.is_glsl_version_supported(&$vers) {
            let _vertex_shader: &str = "";
            let _tessellation_control_shader: Option<&str> = None;
            let _tessellation_evaluation_shader: Option<&str> = None;
            let _geometry_shader: Option<&str> = None;
            let _fragment_shader: &str = "";

            $(
                program!(_program_ty $ty, $src, _vertex_shader, _tessellation_control_shader,
                         _tessellation_evaluation_shader, _geometry_shader, _fragment_shader);
            )+

            let input = $crate::program::ProgramCreationInput::SourceCode {
                vertex_shader: _vertex_shader,
                tessellation_control_shader: _tessellation_control_shader,
                tessellation_evaluation_shader: _tessellation_evaluation_shader,
                geometry_shader: _geometry_shader,
                fragment_shader: _fragment_shader,
                transform_feedback_varyings: None,
            };

            $crate::program::Program::new($context, input)

        } else {
            program!($context, $($rest)*)
        }
    );
    
    (_inner, $context:ident, $vers:ident, {$($ty:ident:$src:expr),+,}$($rest:tt)*) => (
        program!(_inner, $context, $vers, {$($ty:$src),+} $($rest)*);
    );

    (_program_ty vertex, $src:expr, $vs:ident, $tcs:ident, $tes:ident, $gs:ident, $fs:ident) => (
        let $vs = $src;
    );

    (_program_ty tessellation_control, $src:expr, $vs:ident, $tcs:ident, $tes:ident, $gs:ident, $fs:ident) => (
        let $tcs = Some($src);
    );

    (_program_ty tessellation_evaluation, $src:expr, $vs:ident, $tcs:ident, $tes:ident, $gs:ident, $fs:ident) => (
        let $tes = Some($src);
    );

    (_program_ty geometry, $src:expr, $vs:ident, $tcs:ident, $tes:ident, $gs:ident, $fs:ident) => (
        let $gs = Some($src);
    );

    (_program_ty fragment, $src:expr, $vs:ident, $tcs:ident, $tes:ident, $gs:ident, $fs:ident) => (
        let $fs = $src;
    );

    (_parse_num_gl $num:expr) => ($crate::Version($crate::Api::Gl, $num / 100, ($num % 100) / 10));
    (_parse_num_gl 100) => ($crate::Version($crate::Api::GlEs, 1, 0));
    (_parse_num_gles $num:expr) => ($crate::Version($crate::Api::GlEs, $num / 100, ($num % 100) / 10));
}

#[cfg(test)]
mod tests {
    #[test]
    fn trailing_comma_impl_uniforms() {
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
}
