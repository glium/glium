#[derive(PartialEq, Eq)]
enum TextureType {
    Regular,
    Compressed,
    Integral,
    Unsigned,
    Depth,
    Stencil,
    DepthStencil,
}

#[derive(PartialEq, Eq)]
enum TextureDimensions {
    Texture1d,
    Texture2d,
    Texture2dMultisample,
    Texture3d,
    Texture1dArray,
    Texture2dArray,
    Texture2dArrayMultisample,
}

impl TextureDimensions {
    fn is_array(&self) -> bool {
        match self {
            &TextureDimensions::Texture1dArray => true,
            &TextureDimensions::Texture2dArray => true,
            _ => false
        }
    }

    fn is_multisample(&self) -> bool {
        match self {
            &TextureDimensions::Texture2dMultisample => true,
            &TextureDimensions::Texture2dArrayMultisample => true,
            _ => false
        }
    }
}

pub fn build_texture_file<W: Writer>(mut dest: &mut W) {
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2dArrayMultisample);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2dArrayMultisample);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2dArrayMultisample);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2dArrayMultisample);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2dArrayMultisample);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2dArrayMultisample);
}

fn build_texture<W: Writer>(mut dest: &mut W, ty: TextureType, dimensions: TextureDimensions) {
    // building the name of the texture type
    let name: String = {
        let prefix = match ty {
            TextureType::Regular => "",
            TextureType::Compressed => "Compressed",
            TextureType::Integral => "Integral",
            TextureType::Unsigned => "Unsigned",
            TextureType::Depth => "Depth",
            TextureType::Stencil => "Stencil",
            TextureType::DepthStencil => "DepthStencil",
        };

        let suffix = match dimensions {
            TextureDimensions::Texture1d => "Texture1d",
            TextureDimensions::Texture2d => "Texture2d",
            TextureDimensions::Texture2dMultisample => "Texture2dMultisample",
            TextureDimensions::Texture3d => "Texture3d",
            TextureDimensions::Texture1dArray => "Texture1dArray",
            TextureDimensions::Texture2dArray => "Texture2dArray",
            TextureDimensions::Texture2dArrayMultisample => "Texture2dArrayMultisample",
        };

        format!("{}{}", prefix, suffix)
    };

    // the trait corresponding to the data source
    let data_source_trait = match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "Texture1dDataSource",
        TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => "Texture2dDataSource",
        TextureDimensions::Texture3d => "Texture3dDataSource",
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture2dArrayMultisample => {
            "unreachable"
        },
    };

    // the format enum corresponding to this texture
    let relevant_format = match ty {
        TextureType::Regular => "UncompressedFloatFormat",
        TextureType::Compressed => "CompressedFormat",
        TextureType::Integral => "UncompressedIntFormat",
        TextureType::Unsigned => "UncompressedUintFormat",
        TextureType::Depth => "DepthFormat",
        TextureType::Stencil => "StencilFormat",
        TextureType::DepthStencil => "DepthStencilFormat",
    };

    // the default format to use when none is specified
    let default_format = match ty {
        TextureType::Compressed => "TextureFormatRequest::AnyCompressed",
        TextureType::Regular => "TextureFormatRequest::AnyFloatingPoint",
        TextureType::Integral => "TextureFormatRequest::AnyIntegral",
        TextureType::Unsigned => "TextureFormatRequest::AnyUnsigned",
        TextureType::Depth => "TextureFormatRequest::AnyDepth",
        TextureType::Stencil => "TextureFormatRequest::AnyStencil",
        TextureType::DepthStencil => "TextureFormatRequest::AnyDepthStencil",
    };

    // the `#[cfg]` attribute for the related cargo feature
    let cfg_attribute = match ty {
        TextureType::Integral | TextureType::Unsigned => {
            "#[cfg(feature = \"gl_integral_textures\")]"
        },
        TextureType::Depth | TextureType::DepthStencil => {
            "#[cfg(feature = \"gl_depth_textures\")]"
        },
        TextureType::Stencil => "#[cfg(feature = \"gl_stencil_textures\")]",
        _ => ""
    };

    // 
    let dimensions_parameters_input = match dimensions {
        TextureDimensions::Texture1d => "width: u32",
        TextureDimensions::Texture2d => "width: u32, height: u32",
        TextureDimensions::Texture2dMultisample => "width: u32, height: u32, samples: u32",
        TextureDimensions::Texture3d => "width: u32, height: u32, depth: u32",
        TextureDimensions::Texture1dArray => "width: u32, array_size: u32",
        TextureDimensions::Texture2dArray => "width: u32, height: u32, array_size: u32",
        TextureDimensions::Texture2dArrayMultisample => "width: u32, height: u32, array_size: u32, samples: u32",
    };

    let dimensions_parameters_passing = match dimensions {
        TextureDimensions::Texture1d => "width, None, None, None, None",
        TextureDimensions::Texture2d => "width, Some(height), None, None, None",
        TextureDimensions::Texture2dMultisample => "width, Some(height), None, None, Some(samples)",
        TextureDimensions::Texture3d => "width, Some(height), Some(depth), None, None",
        TextureDimensions::Texture1dArray => "width, None, None, Some(array_size), None",
        TextureDimensions::Texture2dArray => "width, Some(height), None, Some(array_size), None",
        TextureDimensions::Texture2dArrayMultisample => "width, Some(height), None, Some(array_size), Some(samples)",
    };

    // writing the struct with doc-comment
    (write!(dest, "/// ")).unwrap();
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture2d |
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture3d => "A ",
        TextureDimensions::Texture1dArray | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dArrayMultisample => "An array of ",
    })).unwrap();
    if ty == TextureType::Compressed {
        (write!(dest, "compressed ")).unwrap();
    }
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "one-dimensional ",
        TextureDimensions::Texture2d | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture2dArrayMultisample => {
            "two-dimensional "
        },
        TextureDimensions::Texture3d => "three-dimensional ",
    })).unwrap();
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture2d |
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture3d => "texture ",
        TextureDimensions::Texture1dArray | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dArrayMultisample => "textures ",
    })).unwrap();
    (write!(dest, "{}", match ty {
        TextureType::Regular | TextureType::Compressed => " containing floating-point data",
        TextureType::Integral => " containing signed integral data",
        TextureType::Unsigned => " containing unsigned integral data",
        TextureType::Depth => " containing depth data",
        TextureType::Stencil => " containing stencil data",
        TextureType::DepthStencil => " containing both depth and stencil data",
    })).unwrap();
    (writeln!(dest, ".")).unwrap();
    (writeln!(dest, "pub struct {}(TextureImplementation);", name)).unwrap();

    // `Texture` trait impl
    (writeln!(dest, "
                impl Texture for {} {{
                    fn get_width(&self) -> u32 {{
                        self.0.get_width()
                    }}

                    fn get_height(&self) -> Option<u32> {{
                        self.0.get_height()
                    }}

                    fn get_depth(&self) -> Option<u32> {{
                        self.0.get_depth()
                    }}

                    fn get_array_size(&self) -> Option<u32> {{
                        self.0.get_array_size()
                    }}
                }}
            ", name)).unwrap();

    // `GlObject` trait impl
    (writeln!(dest, "
                impl GlObject for {} {{
                    type Id = gl::types::GLuint;
                    fn get_id(&self) -> gl::types::GLuint {{
                        self.0.get_id()
                    }}
                }}
            ", name)).unwrap();

    // `Debug` trait impl
    (writeln!(dest, "
                impl ::std::fmt::Debug for {} {{
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error>
                    {{
                        self.0.fmt(f)
                    }}
                }}
            ", name)).unwrap();

    // `UniformValue` trait impl
    {
        match ty {
            TextureType::Regular | TextureType::Compressed |
            TextureType::Integral | TextureType::Unsigned | TextureType::Depth => {
                (writeln!(dest, "
                            impl<'a> IntoUniformValue<'a> for &'a {myname} {{
                                fn into_uniform_value(self) -> UniformValue<'a> {{
                                    UniformValue::{myname}(self, None)
                                }}
                            }}

                            impl<'a> IntoUniformValue<'a> for Sampler<'a, {myname}> {{
                                fn into_uniform_value(self) -> UniformValue<'a> {{
                                    UniformValue::{myname}(self.0, Some(self.1))
                                }}
                            }}
                        ", myname = name)).unwrap();
            },
            _ => ()
        }
    }

    // `ToXXXAttachment` trait impl
    if dimensions == TextureDimensions::Texture2d {
        match ty {
            TextureType::Regular => {
                (writeln!(dest, "
                        impl ::framebuffer::ToColorAttachment for {} {{
                            fn to_color_attachment(&self) -> ::framebuffer::ColorAttachment {{
                                ::framebuffer::ColorAttachment::Texture2d(self.main_level())
                            }}
                        }}
                    ", name)).unwrap();
            },
            TextureType::Depth => {
                (writeln!(dest, "
                        impl ::framebuffer::ToDepthAttachment for {} {{
                            fn to_depth_attachment(&self) -> ::framebuffer::DepthAttachment {{
                                ::framebuffer::DepthAttachment::Texture2d(self.main_level())
                            }}
                        }}
                    ", name)).unwrap();
            },
            TextureType::Stencil => {
                (writeln!(dest, "
                        impl ::framebuffer::ToStencilAttachment for {} {{
                            fn to_stencil_attachment(&self) -> ::framebuffer::StencilAttachment {{
                                ::framebuffer::StencilAttachment::Texture2d(self.main_level())
                            }}
                        }}
                    ", name)).unwrap();
            },
            TextureType::DepthStencil => {
                (writeln!(dest, "
                        impl ::framebuffer::ToDepthStencilAttachment for {} {{
                            fn to_depth_stencil_attachment(&self) -> ::framebuffer::DepthStencilAttachment {{
                                ::framebuffer::DepthStencilAttachment::Texture2d(self.main_level())
                            }}
                        }}
                    ", name)).unwrap();
            },
            _ => ()
        }
    }

    // opening `impl Texture` block
    (writeln!(dest, "impl {} {{", name)).unwrap();

    // writing the `new` function
    if !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                ///
                /// This function will automatically generate all mipmaps of the texture.
                {cfg_attr}
                pub fn new<'a, T>(display: &::Display, data: {param})
                              -> {name} where T: {data_source_trait}<'a>
                {{
                    {name}::new_impl(display, data, None, true).unwrap()
                }}
            ", data_source_trait = data_source_trait, param = param, name = name,
                cfg_attr = cfg_attribute)).unwrap();
    }

    // writing the `new_if_supported` function
    if cfg_attribute.len() >= 1 && !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                ///
                /// This function will automatically generate all mipmaps of the texture.
                pub fn new_if_supported<'a, T>(display: &::Display, data: {param})
                                               -> Option<{name}> where T: {data_source_trait}<'a>
                {{
                    match {name}::new_impl(display, data, None, true) {{
                        Ok(t) => Some(t),
                        Err(TextureMaybeSupportedCreationError::NotSupported) => None,
                        Err(TextureMaybeSupportedCreationError::CreationError(_)) => unreachable!()
                    }}
                }}
            ", data_source_trait = data_source_trait, param = param, name = name)).unwrap();
    }

    // writing the `with_mipmaps` function
    if !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                {cfg_attr}
                pub fn with_mipmaps<'a, T>(display: &::Display, data: {param}, mipmaps: bool)
                                           -> {name} where T: {data_source_trait}<'a>
                {{
                    {name}::new_impl(display, data, None, mipmaps).unwrap()
                }}
            ", data_source_trait = data_source_trait, param = param, name = name,
                cfg_attr = cfg_attribute)).unwrap();
    }

    // writing the `with_mipmaps_if_supported` function
    if cfg_attribute.len() >= 1 && !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                pub fn with_mipmaps_if_supported<'a, T>(display: &::Display, data: {param},
                                                        mipmaps: bool) -> Option<{name}>
                                                        where T: {data_source_trait}<'a>
                {{
                    match {name}::new_impl(display, data, None, mipmaps) {{
                        Ok(t) => Some(t),
                        Err(TextureMaybeSupportedCreationError::NotSupported) => None,
                        Err(TextureMaybeSupportedCreationError::CreationError(_)) => unreachable!()
                    }}
                }}
            ", data_source_trait = data_source_trait, param = param, name = name)).unwrap();
    }

    // writing the `with_format` function
    if !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture with a specific format.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                {cfg_attr}
                pub fn with_format<'a, T>(display: &::Display, data: {param},
                                          format: {format}, mipmaps: bool)
                                          -> Result<{name}, TextureCreationError>
                                          where T: {data_source_trait}<'a>
                {{
                    match {name}::new_impl(display, data, Some(format), mipmaps) {{
                        Ok(t) => Ok(t),
                        Err(TextureMaybeSupportedCreationError::CreationError(e)) => Err(e),
                        Err(TextureMaybeSupportedCreationError::NotSupported) => unreachable!()
                    }}
                }}
            ", data_source_trait = data_source_trait, param = param, cfg_attr = cfg_attribute,
               format = relevant_format, name = name)).unwrap();
    }

    // writing the `with_format_if_supported` function
    if cfg_attribute.len() >= 1 && !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture with a specific format.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                pub fn with_format_if_supported<'a, T>(display: &::Display, data: {param},
                                                       format: {format}, mipmaps: bool)
                                                       -> Result<{name}, TextureMaybeSupportedCreationError>
                                                       where T: {data_source_trait}<'a>
                {{
                    {name}::new_impl(display, data, Some(format), mipmaps)
                }}
            ", data_source_trait = data_source_trait, param = param,
               format = relevant_format, name = name)).unwrap();
    }

    // writing the `new_impl` function
    if !dimensions.is_multisample() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                fn new_impl<'a, T>(display: &::Display, data: {param},
                                   format: Option<{relevant_format}>, mipmaps: bool)
                                   -> Result<{name}, TextureMaybeSupportedCreationError>
                                   where T: {data_source_trait}<'a>
                {{
            ", data_source_trait = data_source_trait,
               param = param,
               name = name,
               relevant_format = relevant_format)).unwrap();

        // writing the `let format = ...` line
        (write!(dest, "let format = format.map(|f| {{
                           TextureFormatRequest::Specific(f.to_texture_format())
                       }}).unwrap_or({});", default_format)).unwrap();

        match dimensions {
            TextureDimensions::Texture1d => (write!(dest, "
                    let RawImage1d {{ data, width, format: client_format }} = data.into_raw();
                ")).unwrap(),

            TextureDimensions::Texture2d => (write!(dest, "
                    let RawImage2d {{ data, width, height, format: client_format }} =
                                            data.into_raw();
                ")).unwrap(),

            TextureDimensions::Texture3d => (write!(dest, "
                    let RawImage3d {{ data, width, height, depth, format: client_format }} =
                                            data.into_raw();
                ")).unwrap(),

            TextureDimensions::Texture1dArray => (write!(dest, "
                    let array_size = 0;
                    let data = Cow::Owned(Vec::<u8>::new());
                    let width = 0;
                    let client_format = unsafe {{ ::std::mem::uninitialized() }};
                    unimplemented!();
                ")).unwrap(),   // TODO: panic if dimensions are inconsistent

            TextureDimensions::Texture2dArray => (write!(dest, "
                    let array_size = 0;
                    let data = Cow::Owned(Vec::<u8>::new());
                    let width = 0;
                    let height = 0;
                    let client_format = unsafe {{ ::std::mem::uninitialized() }};
                    unimplemented!();
                ")).unwrap(),   // TODO: panic if dimensions are inconsistent

            _ => unreachable!()
        }

        // writing the constructor
        (write!(dest, "Ok({}(try!(TextureImplementation::new(display, format, \
                       Some((client_format, data)), mipmaps, {}", name, dimensions_parameters_passing)).unwrap();
        (writeln!(dest, "))))")).unwrap();

        // end of "new" function block
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `new_empty` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture.
                ///
                /// The texture will contain undefined data.
                #[deprecated = \"Use `empty` instead\"]
                pub fn new_empty(display: &::Display, format: {format}, {dim_params}) -> {name} {{
                    let format = format.to_texture_format();
                    let format = TextureFormatRequest::Specific(format);
            ", format = relevant_format, dim_params = dimensions_parameters_input, name = name)).unwrap();

        // writing the constructor
        (write!(dest, "{}(TextureImplementation::new::<u8>(display, format, None, true, {}).unwrap())", name, dimensions_parameters_passing)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture.
                ///
                /// No mipmaps will be created.
                ///
                /// The texture will contain undefined data.
                {cfg_attr}
                pub fn empty(display: &::Display, {dim_params}) -> {name} {{
                    let format = {format};
            ", format = default_format, dim_params = dimensions_parameters_input, name = name,
                cfg_attr = cfg_attribute)).unwrap();

        // writing the constructor
        (write!(dest, "{}(TextureImplementation::new::<u8>(display, format, None, false, {}).unwrap())", name, dimensions_parameters_passing)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_if_supported` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture.
                ///
                /// No mipmaps will be created.
                ///
                /// The texture will contain undefined data.
                pub fn empty_if_supported(display: &::Display, {dim_params}) -> Option<{name}> {{
                    let format = {format};
            ", format = default_format, dim_params = dimensions_parameters_input, name = name)).unwrap();

        // writing the constructor
        (write!(dest, "match TextureImplementation::new::<u8>(display, format, None, false, {})", dimensions_parameters_passing)).unwrap();
        (writeln!(dest, "
            {{
                Ok(t) => Some({}(t)),
                Err(TextureMaybeSupportedCreationError::NotSupported) => None,
                Err(TextureMaybeSupportedCreationError::CreationError(_)) => unreachable!()
            }}", name)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_with_format` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture with a specific format.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                ///
                /// The texture (and its mipmaps, if you pass `true`) will contain undefined data.
                {cfg_attr}
                pub fn empty_with_format(display: &::Display, format: {format}, mipmaps: bool, {dim_params}) -> Result<{name}, TextureCreationError> {{
                    let format = format.to_texture_format();
                    let format = TextureFormatRequest::Specific(format);
            ", format = relevant_format, dim_params = dimensions_parameters_input, name = name,
                cfg_attr = cfg_attribute)).unwrap();

        // writing the constructor
        (write!(dest, "let t = TextureImplementation::new::<u8>(display, format, None, mipmaps, {});", dimensions_parameters_passing)).unwrap();
        (writeln!(dest, "
            match t {{
                Ok(t) => Ok({}(t)),
                Err(TextureMaybeSupportedCreationError::CreationError(e)) => Err(e),
                Err(TextureMaybeSupportedCreationError::NotSupported) => unreachable!()
            }}", name)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_with_format_if_supported` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture with a specific format.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                ///
                /// The texture (and its mipmaps, if you pass `true`) will contain undefined data.
                pub fn empty_with_format_if_supported(display: &::Display, format: {format},
                                                      mipmaps: bool, {dim_params})
                                                      -> Result<{name},
                                                                TextureMaybeSupportedCreationError>
                {{
                    let format = format.to_texture_format();
                    let format = TextureFormatRequest::Specific(format);
            ", format = relevant_format, dim_params = dimensions_parameters_input, name = name)).unwrap();

        // writing the constructor
        (write!(dest, "TextureImplementation::new::<u8>(display, format, None, mipmaps, {})", dimensions_parameters_passing)).unwrap();
        (writeln!(dest, ".map(|t| {}(t))", name)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_with_mipmaps` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture. Specifies whether is has mipmaps.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                ///
                /// The texture (and its mipmaps, if you pass `true`) will contain undefined data.
                {cfg_attr}
                pub fn empty_with_mipmaps(display: &::Display, mipmaps: bool, {dim_params}) -> {name} {{
                    let format = {format};
            ", format = default_format, dim_params = dimensions_parameters_input, name = name,
                cfg_attr = cfg_attribute)).unwrap();

        // writing the constructor
        (write!(dest, "{}(TextureImplementation::new::<u8>(display, format, None, mipmaps, {})", name, dimensions_parameters_passing)).unwrap();
        (writeln!(dest, ".unwrap())")).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_with_mipmaps_if_supported` function
    if ty != TextureType::Compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture. Specifies whether is has mipmaps.
                ///
                /// Note that passing `true` for `mipmaps` does not mean that you will get mipmaps.
                /// Instead it indicates that mipmaps are *allowed* to be created if possible.
                ///
                /// The texture (and its mipmaps, if you pass `true`) will contain undefined data.
                pub fn empty_with_mipmaps_if_supported(display: &::Display, mipmaps: bool,
                                                       {dim_params}) -> Option<{name}> {{
                    let format = {format};
            ", format = default_format, dim_params = dimensions_parameters_input, name = name)).unwrap();

        // writing the constructor
        (write!(dest, "match TextureImplementation::new::<u8>(display, format, None, mipmaps, {})", dimensions_parameters_passing)).unwrap();
        (writeln!(dest, "
            {{
                Ok(t) => Some({}(t)),
                Err(TextureMaybeSupportedCreationError::NotSupported) => None,
                Err(TextureMaybeSupportedCreationError::CreationError(_)) => unreachable!()
            }}", name)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }


    // writing the `as_surface` function
    if dimensions == TextureDimensions::Texture2d && ty == TextureType::Regular {
        (write!(dest, "
                /// Starts drawing on the texture.
                ///
                /// All the function calls to the `TextureSurface` will draw on the texture instead
                /// of the screen.
                ///
                /// ## Low-level information
                ///
                /// The first time that this function is called, a FrameBuffer Object will be
                /// created and cached. The following calls to `as_surface` will load the existing
                /// FBO and re-use it. When the texture is destroyed, the FBO is destroyed too.
                ///
                pub fn as_surface<'a>(&'a self) -> TextureSurface<'a> {{
                    TextureSurface(framebuffer::SimpleFrameBuffer::new(self.0.get_display(), self))
                }}
            ")).unwrap();
    }

    // writing the `get_mipmap_levels` function
    (write!(dest, "
            /// Returns the number of mipmap levels of the texture.
            ///
            /// The minimum value is 1, since there is always a main texture.
            pub fn get_mipmap_levels(&self) -> u32 {{
                self.0.get_mipmap_levels()
            }}
        ")).unwrap();

    // writing the `read` functions
    // TODO: implement for other types too
    if dimensions == TextureDimensions::Texture2d &&
       (ty == TextureType::Regular || ty == TextureType::Compressed)
    {
        (write!(dest, r#"
                /// Reads the content of the texture to RAM.
                ///
                /// You should avoid doing this at all cost during performance-critical
                /// operations (for example, while you're drawing).
                /// Use `read_to_pixel_buffer` instead.
                pub fn read<P, T>(&self) -> T where T: Texture2dDataSink<Data = P>, P: PixelValue + Clone {{
                    self.0.read(0)
                }}
            "#)).unwrap();

        (write!(dest, r#"
                /// Reads the content of the texture into a buffer in video memory.
                ///
                /// This operation copies the texture's data into a buffer in video memory
                /// (a pixel buffer). Contrary to the `read` function, this operation is
                /// done asynchronously and doesn't need a synchronization.
                pub fn read_to_pixel_buffer<P, T>(&self) -> PixelBuffer<T>
                                                  where T: Texture2dDataSink<Data = P>,
                                                        P: PixelValue + Clone
                {{
                    self.0.read_to_pixel_buffer(0)
                }}
            "#)).unwrap();
    }

    // writing the `write` function
    // TODO: implement for other types too
    if dimensions == TextureDimensions::Texture2d &&
       (ty == TextureType::Regular || ty == TextureType::Compressed)
    {
        (write!(dest, r#"
                /// Uploads some data in the texture.
                ///
                /// Note that this may cause a synchronization if you use the texture right before
                /// or right after this call. Prefer creating a whole new texture if you change a
                /// huge part of it.
                ///
                /// ## Panic
                ///
                /// Panics if the the dimensions of `data` don't match the `Rect`.
                pub fn write<'a, T>(&self, rect: Rect, data: T) where T: {data_source_trait}<'a> {{
                    let RawImage2d {{ data, width, height, format: client_format }} =
                                            data.into_raw();

                    assert_eq!(width, rect.width);
                    assert_eq!(height, rect.height);

                    self.0.upload(rect.left, rect.bottom, 0, (client_format, data), width,
                                  Some(height), None, 0, true);
                }}
            "#, data_source_trait = data_source_trait)).unwrap();
    }

    // writing the `layer()` function
    if dimensions.is_array() {
        (write!(dest, r#"
                /// Access a single layer of this texture.
                pub fn layer(&self, layer: u32) -> Option<{name}Layer> {{
                    if layer < self.0.get_array_size().unwrap_or(1) {{
                        Some({name}Layer {{
                            texture: self,
                            layer: layer,
                        }})
                    }} else {{
                        None
                    }}
                }}
            "#, name = name)).unwrap();
    }

    // writing the `mipmap()` and `main_level()` functions
    if !dimensions.is_array() {
        (write!(dest, r#"
                /// Access a single mipmap level of this texture.
                pub fn mipmap(&self, level: u32) -> Option<{name}Mipmap> {{
                    if level < self.0.get_mipmap_levels() {{
                        Some({name}Mipmap {{
                            texture: self,
                            level: level,
                        }})
                    }} else {{
                        None
                    }}
                }}
            "#, name = name)).unwrap();

        (write!(dest, r#"
                /// Access the main mipmap level of this texture.
                pub fn main_level(&self) -> {name}Mipmap {{
                    {name}Mipmap {{
                        texture: self,
                        level: 0,
                    }}
                }}
            "#, name = name)).unwrap();
    }

    // closing `impl Texture` block
    (writeln!(dest, "}}")).unwrap();

    // the `Layer` struct
    if dimensions.is_array() {
        // writing the struct
        (write!(dest, r#"
                /// Represents a single layer of a `{name}`.
                ///
                /// Can be obtained by calling `{name}::layer()`.
                #[derive(Copy, Clone)]
                pub struct {name}Layer<'t> {{
                    texture: &'t {name},
                    layer: u32,
                }}
            "#, name = name)).unwrap();

        // opening `impl Layer` block
        (writeln!(dest, "impl<'t> {}Layer<'t> {{", name)).unwrap();

        // writing the `get_layer` and `get_texture` functions
        (write!(dest, "
                /// Returns the corresponding texture.
                pub fn get_texture(&self) -> &'t {name} {{
                    self.texture
                }}

                /// Returns the layer index.
                pub fn get_layer(&self) -> u32 {{
                    self.layer
                }}
            ", name = name)).unwrap();

        // writing the `get_mipmap_levels` function
        (write!(dest, "
                /// Returns the number of mipmap levels of the texture.
                ///
                /// The minimum value is 1, since there is always a main texture.
                pub fn get_mipmap_levels(&self) -> u32 {{
                    self.texture.get_mipmap_levels()
                }}
            ")).unwrap();

        // writing the `mipmap()` function
        (write!(dest, r#"
                /// Access a single mipmap level of this layer.
                pub fn mipmap(&self, level: u32) -> Option<{name}Mipmap> {{
                    if level < self.texture.get_mipmap_levels() {{
                        Some({name}Mipmap {{
                            texture: self.texture,
                            layer: self.layer,
                            level: level,
                        }})
                    }} else {{
                        None
                    }}
                }}
            "#, name = name)).unwrap();

        // writing the `main_level()` function
        (write!(dest, r#"
                /// Access the main mipmap level of this layer.
                pub fn main_level(&self) -> {name}Mipmap {{
                    {name}Mipmap {{
                        texture: self.texture,
                        layer: self.layer,
                        level: 0,
                    }}
                }}
            "#, name = name)).unwrap();

        // closing `impl Layer` block
        (writeln!(dest, "}}")).unwrap();
    }

    // the `Mipmap` struct
    {
        // writing the struct
        if dimensions.is_array() {
            (write!(dest, r#"
                    /// Represents a single mipmap level of a `{name}`.
                    ///
                    /// Can be obtained by calling `{name}Layer::mipmap()` or
                    /// `{name}Layer::main_level()`.
                    #[derive(Copy, Clone)]
                    pub struct {name}Mipmap<'t> {{
                        texture: &'t {name},
                        layer: u32,
                        level: u32,
                    }}
                "#, name = name)).unwrap();

        } else {
            (write!(dest, r#"
                    /// Represents a single mipmap level of a `{name}`.
                    ///
                    /// Can be obtained by calling `{name}::mipmap()` or `{name}::main_level()`.
                    #[derive(Copy, Clone)]
                    pub struct {name}Mipmap<'t> {{
                        texture: &'t {name},
                        level: u32,
                    }}
                "#, name = name)).unwrap();
        }

        // opening `impl Mipmap` block
        (writeln!(dest, "impl<'t> {}Mipmap<'t> {{", name)).unwrap();

        // writing the `get_levl` and `get_texture` functions
        (write!(dest, "
                /// Returns the corresponding texture.
                pub fn get_texture(&self) -> &'t {name} {{
                    self.texture
                }}

                /// Returns the layer index.
                pub fn get_level(&self) -> u32 {{
                    self.level
                }}
            ", name = name)).unwrap();

        // writing the `get_layer` function
        if dimensions.is_array() {
            (write!(dest, "
                    /// Returns the layer index.
                    pub fn get_layer(&self) -> u32 {{
                        self.layer
                    }}
                ")).unwrap();
        }

        // closing `impl Mipmap` block
        (writeln!(dest, "}}")).unwrap();
    }
}
