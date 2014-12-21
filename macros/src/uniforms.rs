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
    }.expand(ecx, meta_item, item, |i| push(i));
}

fn body(ecx: &mut base::ExtCtxt, span: codemap::Span,
        substr: &generic::Substructure) -> P<ast::Expr>
{
    let ecx: &base::ExtCtxt = ecx;

    match substr.fields {
        &generic::Struct(ref fields) => {
            let mut declarations = Vec::new();

            for (index, field) in fields.iter().enumerate() {
                let ref self_ = field.self_;
                let ident = match field.name {
                    Some(i) => i,
                    None => {
                        ecx.span_err(span, "Unable to implement `glium::uniforms::Uniforms` \
                                            on a structure with unnamed fields");
                        return ecx.expr_int(span, 0);
                    }
                };
                let ident_str = token::get_ident(ident);
                let ident_str = ident_str.get();

                if index == 0 {
                    declarations.push(quote_stmt!(ecx,
                        let uniforms = {
                            use glium::uniforms::UniformsStorage;
                            UniformsStorage::new($ident_str, $self_)
                        };
                    ));

                } else {
                    declarations.push(quote_stmt!(ecx,
                        let uniforms = uniforms.add($ident_str, $self_);
                    ));
                }
            }

            let end = if declarations.len() == 0 {
                quote_expr!(ecx, {
                    use glium::uniforms::EmptyUniforms;
                    EmptyUniforms
                })
            } else {
                quote_expr!(ecx, uniforms)
            };

            quote_expr!(ecx, {
                $declarations
                $end.to_binder()
            })
        },

        _ => {
            ecx.span_err(span, "Unable to implement `glium::uniforms::Uniforms` \
                                on a non-structure");
            ecx.expr_int(span, 0)
        }
    }
}
