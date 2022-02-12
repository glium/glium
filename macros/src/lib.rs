extern crate proc_macro;

// #[cfg(feature = "glslang")]
// extern crate glslang;

// #[cfg(feature = "glslang")]
// mod shaders;

mod uniforms;
mod vertex;

use proc_macro::TokenStream;

// #[cfg(feature = "glslang")]
// #[proc_macro]
// pub fn verify_shader(input: TokenStream) -> TokenStream {
//     shaders::verify_shader(input.into())
//         .unwrap_or_else(syn::Error::into_compile_error)
//         .into()
// }

#[proc_macro_derive(Uniform)]
pub fn uniform(input: TokenStream) -> TokenStream {
    uniforms::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Vertex)]
pub fn vertex(input: TokenStream) -> TokenStream {
    vertex::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
