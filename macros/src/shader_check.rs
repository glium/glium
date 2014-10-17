use glsl_parser;
use std::io::BufReader;
use syntax::ast::TokenTree;
use syntax::ext::base::{mod, DummyResult, ExtCtxt, MacResult, MacExpr};
use syntax::codemap::Span;

pub fn expand(ecx: &mut ExtCtxt, span: Span, input: &[TokenTree]) -> Box<MacResult + 'static> {
    let source = match base::get_single_str_from_tts(ecx, span.clone(), input, "source") {
        Some(s) => s,
        None => return DummyResult::any(span)
    };

    match glsl_parser::parse(BufReader::new(source.as_slice().as_bytes())) {
        Ok(_) => (),
        Err(e) => {
            ecx.span_err(span, format!("error during compile-time check of shader\n{}", e).as_slice());
            return DummyResult::any(span);
        }
    };

    let source = source.as_slice();
    MacExpr::new(quote_expr!(ecx, $source))
}
