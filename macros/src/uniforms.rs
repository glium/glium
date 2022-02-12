use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input: syn::ItemStruct = syn::parse2(input)?;

    let struct_name = &input.ident;

    let statements = {
        let mut statements = Vec::new();

        for field in input.fields.iter() {
            let name = match &field.ident {
                Some(ref name) => Ok(name),
                None => Err(syn::Error::new_spanned(
                    field,
                    "Unable to implement #[uniforms] on structs that have\
                anonymous fields.",
                )),
            }?;
            let name_str = name.to_string();

            statements.push(quote! {
                output(#name_str, ::glium::uniforms::AsUniformValue::as_uniform_value(&self.#name));
            });
        }

        statements
    };

    Ok(quote! {
        impl ::glium::uniforms::Uniforms for #struct_name {
            fn visit_values<'a, F: FnMut(&str, ::glium::uniforms::UniformValue<'a>)>(&'a self, mut output: F) {
                #(#statements)*
            }
        }
    })
}
