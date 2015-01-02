use syntax::ast;
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic;
use syntax::codemap;
use syntax::parse::token;
use syntax::ptr::P;

/// Expand #[vertex_format]
pub fn expand(ecx: &mut base::ExtCtxt, span: codemap::Span,
              meta_item: &ast::MetaItem, item: &ast::Item,
              mut push: Box<FnMut(P<ast::Item>)>)
{
    generic::TraitDef {
        span: span,
        attributes: Vec::new(),
        path: generic::ty::Path {
            path: vec!["glium", "Vertex"],
            lifetime: None,
            params: Vec::new(),
            global: true,
        },
        additional_bounds: Vec::new(),
        generics: generic::ty::LifetimeBounds::empty(),
        methods: vec![
            generic::MethodDef {
                name: "build_bindings",
                generics: generic::ty::LifetimeBounds::empty(),
                explicit_self: None,
                args: vec![
                    generic::ty::Literal(generic::ty::Path {
                        path: vec!["Option"],
                        lifetime: None,
                        params: vec![box generic::ty::Self],
                        global: false,
                    })
                ],
                ret_ty: generic::ty::Literal(
                    generic::ty::Path::new(
                        vec!["glium", "VertexFormat"]
                    ),
                ),
                attributes: vec![
                    ecx.attribute(span.clone(), ecx.meta_list(span.clone(),
                        token::InternedString::new("allow"),
                        vec![ecx.meta_word(span.clone(),
                                token::InternedString::new("unused_assignments"))]
                    ))
                ],
                combine_substructure: generic::combine_substructure(body),
            },
        ],
    }.expand(ecx, meta_item, item, |i| push.call_mut((i,)));
}

fn body(ecx: &mut base::ExtCtxt, span: codemap::Span,
        substr: &generic::Substructure) -> P<ast::Expr>
{
    let ecx: &base::ExtCtxt = ecx;
    let self_ty = &substr.type_ident;

    match substr.fields {
        &generic::StaticStruct(ref definition, generic::Named(ref fields)) => {
            let content = definition.fields.iter().zip(fields.iter())
                .map(|(def, &(ident, _))| {
                    let ref elem_type = def.node.ty;
                    let ident_str = token::get_ident(ident);
                    let ident_str = ident_str.get();

                    quote_expr!(ecx, {
                        let offset = {
                            let dummy: &$self_ty = unsafe { mem::transmute(0u) };
                            let dummy_field = &dummy.$ident;
                            let dummy_field: uint = unsafe { mem::transmute(dummy_field) };
                            dummy_field
                        };

                        bindings.push((
                            $ident_str.to_string(),
                            offset,
                            Attribute::get_type(None::<$elem_type>),
                        ));
                    })

                }).collect::<Vec<P<ast::Expr>>>();

            quote_expr!(ecx, {
                use glium::vertex_buffer::Attribute;
                use std::mem;

                let mut bindings = Vec::new();
                $content;
                bindings
            })
        },

        _ => {
            ecx.span_err(span, "Unable to implement `glium::Vertex::build_bindings` \
                                on a non-structure");
            ecx.expr_int(span, 0)
        }
    }
}
