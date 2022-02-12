use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input: syn::ItemStruct = syn::parse2(input)?;

    let ident = &input.ident;
    let body = body(&input);

    return Ok(quote! {
        impl ::glium::Vertex for #ident {
            #[allow(unused_assignments)]
            fn build_bindings() -> glium::VertexFormat {
                #body
            }
        }
    });
}

fn body(input: &syn::ItemStruct) -> TokenStream {
    let syn::ItemStruct {
        ident: self_ty,
        fields,
        ..
    } = input;

    let content = fields.iter()
        .map(|field| {
            let ref elem_type = field.ty;
            let ident = field.ident.as_ref();
            let ident_str = ident.map(syn::Ident::to_string);

            quote! {
                let offset = {
                    let dummy: &#self_ty = unsafe { mem::transmute(0usize) };
                    let dummy_field = &dummy.#ident;
                    let dummy_field: usize = unsafe { mem::transmute(dummy_field) };
                    dummy_field
                };

                bindings.push((
                    Cow::Borrowed(#ident_str),
                    offset,
                    <#elem_type as Attribute>::get_type(),
                    false,
                ));
            }
        })
        .collect::<Vec<TokenStream>>();

    quote! {
        use glium::vertex::Attribute;
        use std::borrow::Cow;
        use std::mem;

        let mut bindings = Vec::new();
        #(#content)*;
        bindings.into()
    }
}
