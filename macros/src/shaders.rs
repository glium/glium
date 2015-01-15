use glslang;

use syntax::ext::base::{self, MacResult, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ptr::P;
use syntax::parse::token::{self, Token};

pub fn verify_shader<'cx>(ecx: &'cx mut ExtCtxt, span: Span, tts: &[ast::TokenTree])
                          -> Box<MacResult + 'cx>
{
    if tts.len() != 2 {
        ecx.span_err(span, "The verify_shader! macro takes two parameters");
        return base::DummyResult::any(span);
    }

    let ident = match tts[0] {
        ast::TokenTree::TtToken(_, Token::Ident(ref ident, _)) => ident,
        _ => {
            ecx.span_err(span, "Unexpected first parameter. Expected ident.");
            return base::DummyResult::any(span);
        }
    };

    let shader_type = match ident.as_str() {
        "vertex" => glslang::ShaderType::Vertex,
        "fragment" => glslang::ShaderType::Fragment,
        "geometry" => glslang::ShaderType::Geometry,
        "compute" => glslang::ShaderType::Compute,
        "tessellation_control" => glslang::ShaderType::TessellationControl,
        "tessellation_evaluation" => glslang::ShaderType::TessellationEvaluation,
        _ => {
            ecx.span_err(span, "Unexpected first parameter. Must be `vertex`, `fragment`, `geometry`,
                                `compute`, `tessellation_control` or `tessellation_evaluation`.");
            return base::DummyResult::any(span);
        }
    };

    let source = match base::get_single_str_from_tts(ecx, span, tts.slice_from(1), "source code") {
        Some(c) => c,
        None => return base::DummyResult::any(span)
    };

    match glslang::test_shaders(vec![(source.as_slice(), shader_type)]) {
        glslang::TestResult::Ok => (),
        glslang::TestResult::Error(err_str) => {
            ecx.span_err(span, err_str.as_slice());
            return base::DummyResult::any(span);
        },
    };

    base::MacExpr::new(ecx.expr_lit(span, ast::Lit_::LitStr(token::intern_and_get_ident(source.as_slice()), ast::StrStyle::CookedStr)))
}
