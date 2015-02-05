/*!

# verify_shader

*Only available if the `glslang` feature is enabled.*

The `verify_shader!` macro checks that your shader's source code is correct and returns the
source code as an expression.

Example:

```
# #![feature(plugin)]
# #[plugin]
# extern crate glium_macros;
static VERTEX_SHADER: &'static str = verify_shader!(vertex "
    #version 110

    void main() {
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
    }
");
# fn main() {}
```

The first parameter can be `vertex`, `fragment`, `geometry`, `compute`, `tessellation_control`
or `tessellation_evaluation`.

*/
#![feature(core, rustc_private)]
#![feature(plugin_registrar)]
#![feature(quote)]
#![feature(unboxed_closures)]

#[cfg(feature = "glslang")]
extern crate glslang;

extern crate rustc;
extern crate syntax;

#[cfg(feature = "glslang")]
mod shaders;
mod uniforms;
mod vertex;

#[doc(hidden)]
#[plugin_registrar]
pub fn registrar(registry: &mut rustc::plugin::Registry) {
    use syntax::parse::token;

    #[cfg(feature = "glslang")]
    fn register_verify_shader(registry: &mut rustc::plugin::Registry) {
        registry.register_macro("verify_shader", shaders::verify_shader);
    }
    #[cfg(not(feature = "glslang"))]
    fn register_verify_shader(_: &mut rustc::plugin::Registry) {
    }
    register_verify_shader(registry);

    registry.register_syntax_extension(token::intern("uniforms"),
        syntax::ext::base::Decorator(Box::new(uniforms::expand)));
    registry.register_syntax_extension(token::intern("vertex_format"),
        syntax::ext::base::Decorator(Box::new(vertex::expand)));
}

