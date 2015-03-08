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

