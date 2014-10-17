#![feature(plugin_registrar)]
#![feature(quote)]

extern crate glsl_parser;
extern crate rustc;
extern crate syntax;

mod shader_check;
mod uniforms;
mod vertex;

#[doc(hidden)]
#[plugin_registrar]
pub fn registrar(registry: &mut rustc::plugin::Registry) {
    use syntax::parse::token;
    registry.register_syntax_extension(token::intern("uniforms"),
        syntax::ext::base::Decorator(box uniforms::expand));
    registry.register_syntax_extension(token::intern("vertex_format"),
        syntax::ext::base::Decorator(box vertex::expand));
    registry.register_macro("shader_check", shader_check::expand);
}
