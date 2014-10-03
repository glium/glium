use std::fmt;
use std::from_str::FromStr;
use syntax::ast;
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic;
use syntax::{attr, codemap};
use syntax::parse::token;
use syntax::ptr::P;

/// Expand #[vertex_format]
pub fn expand(ecx: &mut base::ExtCtxt, span: codemap::Span,
              meta_item: &ast::MetaItem, item: &ast::Item,
              push: |P<ast::Item>|)
{
    generic::TraitDef {
        span: span,
        attributes: Vec::new(),
        path: generic::ty::Path {
            path: vec!["glium_core", "VertexFormat"],
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
                        vec!["glium_core", "VertexBindings"]
                    ),
                ),
                attributes: vec![
                    ecx.attribute(span.clone(), ecx.meta_list(span.clone(),
                        token::InternedString::new("allow"),
                        vec![ecx.meta_word(span.clone(),
                                token::InternedString::new("dead_assignment"))]
                    ))
                ],
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
        &generic::StaticStruct(ref definition, generic::Named(ref fields)) => {
            let content = definition.fields.iter().zip(fields.iter())
                .map(|(def, &(ident, _))| {
                    let ref elem_type = def.node.ty;
                    let ident_str = token::get_ident(ident);
                    let ident_str = ident_str.get();

                    quote_expr!(ecx, {
                        bindings.insert($ident_str.to_string(), (
                            GLDataTuple::get_gl_type(None::<$elem_type>),
                            GLDataTuple::get_num_elems(None::<$elem_type>),
                            offset_sum
                        ));

                        offset_sum += mem::size_of::<$elem_type>();
                    })

                }).collect::<Vec<P<ast::Expr>>>();

            quote_expr!(ecx, {
                use glium_core::GLDataTuple;
                use std::mem;

                let mut bindings = { use std::collections::HashMap; HashMap::new() };
                let mut offset_sum = 0;
                $content;
                bindings
            })
        },

        _ => {
            ecx.span_err(span, "Unable to implement `glium_core::VertexFormat::build_bindings` \
                                on a non-structure");
            ecx.expr_lit(span, ast::LitNil)
        }
    }
}
