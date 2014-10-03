#![feature(plugin_registrar)]
#![feature(quote)]

extern crate rustc;
extern crate syntax;

mod vertex;

#[doc(hidden)]
#[plugin_registrar]
pub fn registrar(registry: &mut rustc::plugin::Registry) {
    use syntax::parse::token;
    registry.register_syntax_extension(token::intern("vertex_format"),
        syntax::ext::base::Decorator(box vertex::expand));
}
