use syntax::ast;
use syntax::ext::base;
use syntax::codemap;
use syntax::ptr::P;

/// Expand #[uniforms]
pub fn expand(ecx: &mut base::ExtCtxt, span: codemap::Span,
              _meta_item: &ast::MetaItem, item: &ast::Item,
              mut push: Box<FnMut(P<ast::Item>)>)
{
    let struct_name = &item.ident;

    let (struct_def, struct_generics) = match &item.node {
        &ast::ItemStruct(ref struct_def, ref generics) => (struct_def, generics),
        _ => {
            ecx.span_err(span, "Unable to implement #[uniforms] on anything else than a struct.");
            return;
        }
    };

    let statements = {
        let mut statements = Vec::new();

        for field in struct_def.fields.iter() {
            let ref field = field.node;

            let name = match field.kind {
                ast::StructFieldKind::NamedField(name, _) => name,
                _ => {
                    ecx.span_err(span, "Unable to implement #[uniforms] on structs that have\
                                        anonymous fields.");
                    return;
                }
            };

            let name_str = name.as_str();

            statements.push(quote_stmt!(ecx,
                output($name_str, &::glium::uniforms::IntoUniformValue::into_uniform_value(self.$name));
            ));
        }

        statements
    };

    // implementing "Copy" if necessary
    // TODO: VERY HACKY
    {
        let mut found = false;

        for attr in item.attrs.iter() {
            let ref attr = attr.node;
            let src = ::syntax::ext::quote::rt::ToSource::to_source(attr);

            if src.as_slice().contains("deriving") && src.as_slice().contains("Copy") {
                found = true;
                break;
            }
        }

        if !found {
            push.call_mut((quote_item!(ecx,
                impl $struct_generics Copy for $struct_name $struct_generics {}
            ).unwrap(),));
        }
    }

    push.call_mut((quote_item!(ecx,
        impl $struct_generics ::glium::uniforms::Uniforms for $struct_name $struct_generics {
            fn visit_values<F: FnMut(&str, &::glium::uniforms::UniformValue)>(self, mut output: F) {
                $statements
            }
        }
    ).unwrap(),));
}
