use syntax::ast;
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic;
use syntax::codemap;
use syntax::parse::token;
use syntax::ptr::P;

/// Expand #[uniforms]
pub fn expand(ecx: &mut base::ExtCtxt, span: codemap::Span,
              meta_item: &ast::MetaItem, item: &ast::Item,
              push: |P<ast::Item>|)
{
    generic::TraitDef {
        span: span,
        attributes: Vec::new(),
        path: generic::ty::Path {
            path: vec!["glium", "uniforms", "Uniforms"],
            lifetime: None,
            params: Vec::new(),
            global: true,
        },
        additional_bounds: Vec::new(),
        generics: generic::ty::LifetimeBounds::empty(),
        methods: vec![
            generic::MethodDef {
                name: "to_binder",
                generics: generic::ty::LifetimeBounds::empty(),
                explicit_self: generic::ty::borrowed_explicit_self(),
                args: vec![],
                ret_ty: generic::ty::Literal(
                    generic::ty::Path::new(
                        vec!["glium", "uniforms", "UniformsBinder"]
                    ),
                ),
                attributes: vec![],
                combine_substructure: generic::combine_substructure(body),
            },
        ],
    }.expand(ecx, meta_item, item, push);
}

fn body(ecx: &mut base::ExtCtxt, span: codemap::Span,
        substr: &generic::Substructure) -> P<ast::Expr>
{
    let ecx: &base::ExtCtxt = ecx;

    match substr.fields {
        &generic::Struct(ref fields) => {
            let mut declarations = Vec::new();
            let mut linkers = Vec::new();

            for field in fields.iter() {
                let ref self_ = field.self_;
                let ident = match field.name {
                    Some(i) => i,
                    None => {
                        ecx.span_err(span, "Unable to implement `glium::uniforms::Uniforms` \
                                            on a structure with unnamed fields");
                        return ecx.expr_lit(span, ast::LitNil);
                    }
                };
                let ident_str = token::get_ident(ident);
                let ident_str = ident_str.get();

                declarations.push(quote_stmt!(ecx,
                    let $ident = $self_.to_binder();
                ));

                linkers.push(quote_expr!(ecx, {
                    unsafe { gl.ActiveTexture(active_texture as u32) };

                    match loc_getter($ident_str) {
                        Some(loc) => {
                            let v = $ident.get_proc();
                            v(gl, loc, &mut active_texture);
                        },
                        None => ()
                    };
                }));
            }

            quote_expr!(ecx, {
                use glium::uniforms::UniformValue;

                $declarations

                ::glium::uniforms::UniformsBinder(proc(gl, loc_getter) {
                    let mut active_texture = 0x84C0; // gl::TEXTURE0
                    $linkers
                })
            })
        },

        _ => {
            ecx.span_err(span, "Unable to implement `glium::uniforms::Uniforms` \
                                on a non-structure");
            ecx.expr_lit(span, ast::LitNil)
        }
    }
}
