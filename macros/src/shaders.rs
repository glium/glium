use proc_macro2::TokenStream;

struct Input {
    ident: syn::Ident,
    source: syn::LitStr,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Input {
            ident: input.parse()?,
            source: input.parse()?,
        })
    }
}

pub fn verify_shader<'cx>(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let Input { ident, source: shader } = syn::parse_macro_input!(input as Input)?;

    let shader_type = match ident.as_str() {
        "vertex" => glslang::ShaderType::Vertex,
        "fragment" => glslang::ShaderType::Fragment,
        "geometry" => glslang::ShaderType::Geometry,
        "compute" => glslang::ShaderType::Compute,
        "tessellation_control" => glslang::ShaderType::TessellationControl,
        "tessellation_evaluation" => glslang::ShaderType::TessellationEvaluation,
        _ => {
            return Err(syn::Error::new_spanned(ident, 
                "Unexpected first parameter. Must be `vertex`, `fragment`, `geometry`, `compute`, `tessellation_control` or `tessellation_evaluation`.",
            ));
        }
    };

    match glslang::test_shaders(vec![(source.as_slice(), shader_type)]) {
        glslang::TestResult::Ok => (),
        glslang::TestResult::Error(err_str) => {
            return Err(syn::Error::new_spanned(source, err_str        ));
        }
    };

    Ok(source)
}
