use std::io::Write;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TextureType {
    Regular,
    Compressed,
    Srgb,
    CompressedSrgb,
    Integral,
    Unsigned,
    Depth,
    Stencil,
    DepthStencil,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TextureDimensions {
    Texture1d,
    Texture2d,
    Texture2dMultisample,
    Texture3d,
    Texture1dArray,
    Texture2dArray,
    Texture2dMultisampleArray,
    Cubemap,
    CubemapArray,
}

impl TextureDimensions {
    fn is_array(&self) -> bool {
        match self {
            &TextureDimensions::Texture1dArray => true,
            &TextureDimensions::Texture2dArray => true,
            &TextureDimensions::Texture2dMultisampleArray => true,
            &TextureDimensions::CubemapArray => true,
            _ => false
        }
    }

    fn is_multisample(&self) -> bool {
        match self {
            &TextureDimensions::Texture2dMultisample => true,
            &TextureDimensions::Texture2dMultisampleArray => true,
            _ => false
        }
    }

    fn is_cube(&self) -> bool {
        match self {
            &TextureDimensions::Cubemap => true,
            &TextureDimensions::CubemapArray => true,
            _ => false
        }
    }
}

pub fn build_texture_file<W: Write>(dest: &mut W) {
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture1d);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2d);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2dMultisample);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture3d);
    //build_texture(dest, TextureType::Stencil, TextureDimensions::Texture3d);  // forbidden
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture3d);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture1dArray);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2dArray);
    build_texture(dest, TextureType::Regular, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::Integral, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::Depth, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Texture2dMultisampleArray);
    build_texture(dest, TextureType::Regular, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Compressed, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Srgb, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Integral, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Depth, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Stencil, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::Cubemap);
    build_texture(dest, TextureType::Regular, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::Compressed, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::Srgb, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::CompressedSrgb, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::Integral, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::Unsigned, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::Depth, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::Stencil, TextureDimensions::CubemapArray);
    build_texture(dest, TextureType::DepthStencil, TextureDimensions::CubemapArray);
}

fn build_texture<W: Write>(dest: &mut W, ty: TextureType, dimensions: TextureDimensions) {
    // building the name of the module
    let module_name: String = {
        let prefix = match ty {
            TextureType::Regular => "",
            TextureType::Compressed => "compressed_",
            TextureType::Srgb => "srgb_",
            TextureType::CompressedSrgb => "compressed_srgb_",
            TextureType::Integral => "integral_",
            TextureType::Unsigned => "unsigned_",
            TextureType::Depth => "depth_",
            TextureType::Stencil => "stencil_",
            TextureType::DepthStencil => "depth_stencil_",
        };

        let suffix = match dimensions {
            TextureDimensions::Texture1d => "texture1d",
            TextureDimensions::Texture2d => "texture2d",
            TextureDimensions::Texture2dMultisample => "texture2d_multisample",
            TextureDimensions::Texture3d => "texture3d",
            TextureDimensions::Texture1dArray => "texture1d_array",
            TextureDimensions::Texture2dArray => "texture2d_array",
            TextureDimensions::Texture2dMultisampleArray => "texture2d_multisample_array",
            TextureDimensions::Cubemap => "cubemap",
            TextureDimensions::CubemapArray => "cubemap_array",
        };

        format!("{}{}", prefix, suffix)
    };

    // building the name of the texture type
    let name: String = {
        let prefix = match ty {
            TextureType::Regular => "",
            TextureType::Compressed => "Compressed",
            TextureType::Srgb => "Srgb",
            TextureType::CompressedSrgb => "CompressedSrgb",
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
            TextureDimensions::Texture2dMultisampleArray => "Texture2dMultisampleArray",
            TextureDimensions::Cubemap => "Cubemap",
            TextureDimensions::CubemapArray => "CubemapArray",
        };

        format!("{}{}", prefix, suffix)
    };

    // the trait corresponding to the data source
    let data_source_trait = match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "Texture1dDataSource",
        TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => "Texture2dDataSource",
        TextureDimensions::Texture3d => "Texture3dDataSource",
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture2dMultisampleArray |
        TextureDimensions::Cubemap | TextureDimensions::CubemapArray => {
            "unreachable"
        },
    };

    // the format enum corresponding to this texture
    let relevant_format = match ty {
        TextureType::Regular => "UncompressedFloatFormat",
        TextureType::Compressed => "CompressedFormat",
        TextureType::Srgb => "SrgbFormat",
        TextureType::CompressedSrgb => "CompressedSrgbFormat",
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
        TextureType::CompressedSrgb => "TextureFormatRequest::AnyCompressedSrgb",
        TextureType::Srgb => "TextureFormatRequest::AnySrgb",
        TextureType::Integral => "TextureFormatRequest::AnyIntegral",
        TextureType::Unsigned => "TextureFormatRequest::AnyUnsigned",
        TextureType::Depth => "TextureFormatRequest::AnyDepth",
        TextureType::Stencil => "TextureFormatRequest::AnyStencil",
        TextureType::DepthStencil => "TextureFormatRequest::AnyDepthStencil",
    };

    // whether this is a internally compressed texture object
    let is_compressed = match ty {
        TextureType::Compressed |
        TextureType::CompressedSrgb => true,
        _ => false,
    };

    let client_format_any_ty = match ty {
        TextureType::Compressed => "ClientFormatAny::CompressedFormat",
        TextureType::CompressedSrgb => "ClientFormatAny::CompressedSrgbFormat",
        _ => "ClientFormatAny::ClientFormat",
    };

    let mipmaps_option_ty = match ty {
        TextureType::Compressed | TextureType::CompressedSrgb => "CompressedMipmapsOption",
        _ => "MipmapsOption",
    };

    let mipmap_default = match ty {
        TextureType::Compressed | TextureType::CompressedSrgb => "CompressedMipmapsOption::NoMipmap",
        TextureType::Unsigned | TextureType::Integral => "MipmapsOption::NoMipmap",
        _ => "MipmapsOption::AutoGeneratedMipmaps",
    };

    //
    let dimensions_parameters_input = match dimensions {
        TextureDimensions::Texture1d => "width: u32",
        TextureDimensions::Texture2d => "width: u32, height: u32",
        TextureDimensions::Texture2dMultisample => "width: u32, height: u32, samples: u32",
        TextureDimensions::Texture3d => "width: u32, height: u32, depth: u32",
        TextureDimensions::Texture1dArray => "width: u32, array_size: u32",
        TextureDimensions::Texture2dArray => "width: u32, height: u32, array_size: u32",
        TextureDimensions::Texture2dMultisampleArray => "width: u32, height: u32, array_size: u32, samples: u32",
        TextureDimensions::Cubemap => "dimension: u32",
        TextureDimensions::CubemapArray => "dimension: u32, array_size: u32",
    };

    let dimensions_parameters_passing = match dimensions {
        TextureDimensions::Texture1d => {
            "Dimensions::Texture1d { width: width }"
        },
        TextureDimensions::Texture2d => {
            "Dimensions::Texture2d { width: width, height: height }"
        },
        TextureDimensions::Texture2dMultisample => {
            "Dimensions::Texture2dMultisample { width: width, height: height, samples: samples }"
        },
        TextureDimensions::Texture3d => {
            "Dimensions::Texture3d { width: width, height: height, depth: depth }"
        },
        TextureDimensions::Texture1dArray => {
            "Dimensions::Texture1dArray { width: width, array_size: array_size }"
        },
        TextureDimensions::Texture2dArray => {
            "Dimensions::Texture2dArray { width: width, height: height, array_size: array_size }"
        },
        TextureDimensions::Texture2dMultisampleArray => {
            "Dimensions::Texture2dMultisampleArray { width: width, height: height, array_size: array_size, samples: samples }"
        },
        TextureDimensions::Cubemap => {
            "Dimensions::Cubemap { dimension: dimension }"
        },
        TextureDimensions::CubemapArray => {
            "Dimensions::CubemapArray { dimension: dimension, array_size: array_size }"
        },
    };

    // writing the `use module::*;` statement
    writeln!(dest, "pub use self::{}::{};", module_name, name).unwrap();

    // opening `mod module {`
    writeln!(dest, "
        /// Contains the implementation of `{}`.
        pub mod {} {{\
            // the list of imports we need depends on the texture type, don't bother with this
            #![allow(unused_imports)]

            use std::borrow::Cow;

            use texture::any::{{self, TextureAny, TextureAnyLayer, TextureAnyMipmap}};
            use texture::any::{{TextureAnyLayerMipmap, TextureAnyImage, Dimensions}};
            use texture::bindless::{{ResidentTexture, BindlessTexturesNotSupportedError}};
            use texture::get_format::{{InternalFormat, InternalFormatType, GetFormatError}};
            use texture::pixel_buffer::PixelBuffer;
            use texture::{{TextureCreationError, Texture1dDataSource, Texture2dDataSource}};
            use texture::{{Texture3dDataSource, Texture2dDataSink, MipmapsOption, CompressedMipmapsOption}};
            use texture::{{RawImage1d, RawImage2d, RawImage3d, CubeLayer}};
            use texture::pixel::PixelValue;

            use image_format::{{ClientFormatAny, TextureFormatRequest}};
            use image_format::{{UncompressedFloatFormat, UncompressedIntFormat}};
            use image_format::{{CompressedFormat, DepthFormat, DepthStencilFormat, StencilFormat}};
            use image_format::{{CompressedSrgbFormat, SrgbFormat, UncompressedUintFormat}};

            use backend::Facade;
            use uniforms::{{UniformValue, AsUniformValue, Sampler}};
            use framebuffer;
            use Rect;

            use GlObject;
            use TextureExt;
            use TextureMipmapExt;
            use gl;

    ", name, module_name).unwrap();

    // writing the struct with doc-comment
    (write!(dest, "/// ")).unwrap();
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture2d |
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture3d |
        TextureDimensions::Cubemap => "A ",
        TextureDimensions::Texture1dArray | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dMultisampleArray |
        TextureDimensions::CubemapArray => "An array of ",
    })).unwrap();
    if is_compressed {
        (write!(dest, "compressed ")).unwrap();
    }
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "one-dimensional ",
        TextureDimensions::Texture2d | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture2dMultisampleArray => {
            "two-dimensional "
        },
        TextureDimensions::Texture3d => "three-dimensional ",
        TextureDimensions::Cubemap | TextureDimensions::CubemapArray => "cube ",
    })).unwrap();
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture2d |
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture3d |
        TextureDimensions::Cubemap => "texture ",
        TextureDimensions::Texture1dArray | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dMultisampleArray |
        TextureDimensions::CubemapArray => "textures ",
    })).unwrap();
    (write!(dest, "{}", match ty {
        TextureType::Regular | TextureType::Compressed => " containing floating-point data",
        TextureType::Srgb | TextureType::CompressedSrgb => " containing sRGB floating-point data",
        TextureType::Integral => " containing signed integral data",
        TextureType::Unsigned => " containing unsigned integral data",
        TextureType::Depth => " containing depth data",
        TextureType::Stencil => " containing stencil data",
        TextureType::DepthStencil => " containing both depth and stencil data",
    })).unwrap();
    (writeln!(dest, ".")).unwrap();
    (writeln!(dest, "pub struct {}(TextureAny);", name)).unwrap();

    // `GlObject` trait impl
    (writeln!(dest, "
                impl GlObject for {} {{
                    type Id = gl::types::GLuint;

                    #[inline]
                    fn get_id(&self) -> gl::types::GLuint {{
                        self.0.get_id()
                    }}
                }}
            ", name)).unwrap();

    // `Debug` trait impl
    (writeln!(dest, "
                impl ::std::fmt::Debug for {} {{
                    #[inline]
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error>
                    {{
                        self.0.fmt(f)
                    }}
                }}
            ", name)).unwrap();

    // 'Deref' impl to common type.
    (writeln!(dest, "
                impl ::std::ops::Deref for {} {{
                    type Target = TextureAny;

                    #[inline]
                    fn deref<'a>(&'a self) -> &'a TextureAny {{
                        &self.0
                    }}
                }}
            ", name)).unwrap();

    // `UniformValue` trait impl
    {
        match ty {
            TextureType::Regular | TextureType::Compressed |
            TextureType::Srgb | TextureType::CompressedSrgb |
            TextureType::Integral | TextureType::Unsigned | TextureType::Depth => {
                (writeln!(dest, "
                            impl<'a> AsUniformValue for &'a {myname} {{
                                #[inline]
                                fn as_uniform_value(&self) -> UniformValue {{
                                    UniformValue::{myname}(*self, None)
                                }}
                            }}

                            impl<'a> AsUniformValue for Sampler<'a, {myname}> {{
                                #[inline]
                                fn as_uniform_value(&self) -> UniformValue {{
                                    UniformValue::{myname}(self.0, Some(self.1))
                                }}
                            }}

                            impl {myname} {{
                                /// Builds a `Sampler` marker object that allows you to indicate
                                /// how the texture should be sampled from inside a shader.
                                ///
                                /// # Example
                                ///
                                /// ```no_run
                                /// # #[macro_use] extern crate glium;
                                /// # fn main() {{
                                /// # let texture: glium::texture::Texture2d = unsafe {{
                                /// # ::std::mem::uninitialized() }};
                                /// let uniforms = uniform! {{
                                ///     color_texture: texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                                /// }};
                                /// # }}
                                /// ```
                                #[inline]
                                pub fn sampled(&self) -> Sampler<{myname}> {{
                                    Sampler(self, Default::default())
                                }}
                            }}
                        ", myname = name)).unwrap();
            },
            _ => ()
        }
    }

    // `ToXXXAttachment` trait impl
    if dimensions == TextureDimensions::Texture2d || dimensions == TextureDimensions::Texture2dMultisample ||
       dimensions == TextureDimensions::Texture1d
    {
        match ty {
            TextureType::Regular | TextureType::Srgb | TextureType::Integral | TextureType::Unsigned => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToColorAttachment<'t> for &'t {name} {{
                            #[inline]
                            fn to_color_attachment(self) -> ::framebuffer::ColorAttachment<'t> {{
                                ::framebuffer::ColorAttachment::Texture(self.0.main_level().first_layer().into_image(None).unwrap())
                            }}
                        }}
                    ", name = name)).unwrap();
            },
            TextureType::Depth => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToDepthAttachment<'t> for &'t {name} {{
                            #[inline]
                            fn to_depth_attachment(self) -> ::framebuffer::DepthAttachment<'t> {{
                                ::framebuffer::DepthAttachment::Texture(self.0.main_level().first_layer().into_image(None).unwrap())
                            }}
                        }}
                    ", name = name)).unwrap();
            },
            TextureType::Stencil => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToStencilAttachment<'t> for &'t {name} {{
                            #[inline]
                            fn to_stencil_attachment(self) -> ::framebuffer::StencilAttachment<'t> {{
                                ::framebuffer::StencilAttachment::Texture(self.0.main_level().first_layer().into_image(None).unwrap())
                            }}
                        }}
                    ", name = name)).unwrap();
            },
            TextureType::DepthStencil => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToDepthStencilAttachment<'t> for &'t {name} {{
                            #[inline]
                            fn to_depth_stencil_attachment(self) -> ::framebuffer::DepthStencilAttachment<'t> {{
                                ::framebuffer::DepthStencilAttachment::Texture(self.0.main_level().first_layer().into_image(None).unwrap())
                            }}
                        }}
                    ", name = name)).unwrap();
            },
            _ => ()
        }
    }

    // opening `impl Texture` block
    (writeln!(dest, "impl {} {{", name)).unwrap();

    // writing `get_internal_format`
    writeln!(dest, "
            /// Determines the internal format of this texture.
            ///
            /// The backend may not support querying the actual format, in which case an error
            /// is returned.
            #[inline]
            pub fn get_internal_format(&self) -> Result<InternalFormat, GetFormatError> {{
                self.0.get_internal_format()
            }}
        ").unwrap();

    // writing the `new` function
    if !dimensions.is_multisample() && !dimensions.is_cube() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        let gen_doc = if is_compressed {
            "/// No mipmap level (except for the main level) will be allocator nor generated."
        } else {
            "/// This function will automatically generate all mipmaps of the texture."
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                ///
                {gen_doc}
                #[inline]
                pub fn new<'a, F: ?Sized, T>(facade: &F, data: {param})
                              -> Result<{name}, TextureCreationError>
                              where T: {data_source_trait}<'a>, F: Facade
                {{
                    {name}::new_impl(facade, data, None, {mipmap_default})
                }}
            ", data_source_trait = data_source_trait, param = param, name = name,
               mipmap_default = mipmap_default, gen_doc = gen_doc)).unwrap();
    }

    // writing the `with_mipmaps` function
    if !dimensions.is_multisample() && !dimensions.is_cube() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                #[inline]
                pub fn with_mipmaps<'a, F: ?Sized, T>(facade: &F, data: {param}, mipmaps: {mipmaps})
                                              -> Result<{name}, TextureCreationError>
                                              where T: {data_source_trait}<'a>, F: Facade
                {{
                    {name}::new_impl(facade, data, None, mipmaps)
                }}
            ", data_source_trait = data_source_trait, param = param, name = name,
               mipmaps = mipmaps_option_ty)).unwrap();
    }

    // writing the `with_compressed_data` function
    if is_compressed && !dimensions.is_multisample() && !dimensions.is_cube() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "&[u8]",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<&[u8]>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture with a specific format. The input data must also be of the
                /// specified compressed format.
                #[inline]
                pub fn with_compressed_data<F: ?Sized>(facade: &F, data: {param}, {dim_params},
                                                      format: {format}, mipmaps: {mipmaps})
                                                      -> Result<{name}, TextureCreationError>
                                                       where F: Facade
                {{
                    let data = Cow::Borrowed(data.as_ref());
                    let client_format = {client_format_any}(format);
                    Ok({name}(any::new_texture(facade, {default_format}, Some((client_format, data)),
                                                    mipmaps.into(), {dim_params_passing})?))
                }}
            ", dim_params = dimensions_parameters_input, dim_params_passing = dimensions_parameters_passing,
               param = param, client_format_any = client_format_any_ty,
               name = name, format = relevant_format, default_format = default_format,
               mipmaps = mipmaps_option_ty).unwrap());
    }

    // writing the `with_format` function
    if !dimensions.is_multisample() && !dimensions.is_cube() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                /// Builds a new texture with a specific format.
                #[inline]
                pub fn with_format<'a, F: ?Sized, T>(facade: &F, data: {param},
                                          format: {format}, mipmaps: {mipmaps})
                                          -> Result<{name}, TextureCreationError>
                                          where T: {data_source_trait}<'a>, F: Facade
                {{
                    {name}::new_impl(facade, data, Some(format), mipmaps)
                }}
            ", data_source_trait = data_source_trait, param = param,
               format = relevant_format, name = name,
               mipmaps = mipmaps_option_ty)).unwrap();
    }

    // writing the `new_impl` function
    if !dimensions.is_multisample() && !dimensions.is_cube() {
        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",

            _ => unreachable!()
        };

        (writeln!(dest, "
                #[inline]
                fn new_impl<'a, F: ?Sized, T>(facade: &F, data: {param},
                                   format: Option<{relevant_format}>, mipmaps: {mipmaps})
                                   -> Result<{name}, TextureCreationError>
                                   where T: {data_source_trait}<'a>, F: Facade
                {{
            ", data_source_trait = data_source_trait,
               param = param, name = name,
               relevant_format = relevant_format,
               mipmaps = mipmaps_option_ty)).unwrap();

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
                    let vec_raw = data.into_iter().map(|e| e.into_raw()).collect();
                    let RawImage2d {{data, width, height: array_size, format: client_format }} = RawImage2d::from_vec_raw1d(&vec_raw);
                ")).unwrap(),   // TODO: panic if dimensions are inconsistent

            TextureDimensions::Texture2dArray => (write!(dest, "
                    let vec_raw = data.into_iter().map(|e| e.into_raw()).collect();
                    let RawImage3d {{data, width, height, depth: array_size, format: client_format }} = RawImage3d::from_vec_raw2d(&vec_raw);
                ")).unwrap(),   // TODO: panic if dimensions are inconsistent

            _ => unreachable!()
        }

        (write!(dest, "let client_format = ClientFormatAny::ClientFormat(client_format);")).unwrap();

        // writing the constructor
        (write!(dest, "Ok({}(any::new_texture(facade, format, \
                       Some((client_format, data)), mipmaps.into(), {}", name, dimensions_parameters_passing)).unwrap();
        (writeln!(dest, ")?))")).unwrap();

        // end of "new" function block
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty` function
    if !is_compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture.
                ///
                /// No mipmap level (except for the main level) will be allocated or generated.
                ///
                /// The texture will contain undefined data.
                #[inline]
                pub fn empty<F: ?Sized>(facade: &F, {dim_params})
                                -> Result<{name}, TextureCreationError>
                                where F: Facade
                {{
                    let format = {format};
            ", format = default_format, dim_params = dimensions_parameters_input, name = name,
               )).unwrap();

        // writing the constructor
        (write!(dest, "any::new_texture::<_, u8>(facade, format, None, {mipmap}::NoMipmap.into(), {}).map(|t| {}(t))",
                dimensions_parameters_passing, name, mipmap = mipmaps_option_ty)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_with_format` function
    if true {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture with a specific format.
                ///
                /// The texture (and its mipmaps) will contain undefined data.
                #[inline]
                pub fn empty_with_format<F: ?Sized>(facade: &F, format: {format}, mipmaps: {mipmaps}, {dim_params}) -> Result<{name}, TextureCreationError> where F: Facade {{
                    let format = format.to_texture_format();
                    let format = TextureFormatRequest::Specific(format);
            ", format = relevant_format, dim_params = dimensions_parameters_input, name = name,
               mipmaps = mipmaps_option_ty)).unwrap();

        // writing the constructor
        (write!(dest, "let t = any::new_texture::<_, u8>(facade, format, None, mipmaps.into(), {});", dimensions_parameters_passing)).unwrap();
        (writeln!(dest, "
            t.map(|t| {}(t))", name)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `empty_with_mipmaps` function
    if !is_compressed {
        // opening function
        (writeln!(dest, "
                /// Creates an empty texture. Specifies whether is has mipmaps.
                ///
                /// The texture (and its mipmaps) will contain undefined data.
                #[inline]
                pub fn empty_with_mipmaps<F: ?Sized>(facade: &F, mipmaps: {mipmaps}, {dim_params}) -> Result<{name}, TextureCreationError> where F: Facade {{
                    let format = {format};
            ", format = default_format, dim_params = dimensions_parameters_input, name = name,
               mipmaps = mipmaps_option_ty)).unwrap();

        // writing the constructor
        (write!(dest, "any::new_texture::<_, u8>(facade, format, None, mipmaps.into(), {})", dimensions_parameters_passing)).unwrap();
        (writeln!(dest, ".map(|t| {}(t))", name)).unwrap();

        // closing function
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the 'from_id' function
    (writeln!(dest, "
                /// Builds a new texture reference from an existing, externally created OpenGL texture.
                /// If `owned` is true, this reference will take ownership of the texture and be responsible
                /// for cleaning it up. Otherwise, the texture must be cleaned up externally, but only
                /// after this reference's lifetime has ended.
                pub unsafe fn from_id<F: Facade + ?Sized>(facade: &F,
                                                 format: {format},
                                                 id: gl::types::GLuint,
                                                 owned: bool,
                                                 mipmap: MipmapsOption,
                                                 ty: Dimensions)
                                                 -> {name} {{
                    let format = format.to_texture_format();
                    let format = TextureFormatRequest::Specific(format);
                    {name}(any::from_id(facade, format, id, owned, mipmap, ty))
                }}
        ", format = relevant_format, name = name)).unwrap();

    // dimensions getters
    write_dimensions_getters(dest, dimensions, "self.0", true);

    // writing the `as_surface` function
    if (dimensions == TextureDimensions::Texture2d ||
        dimensions == TextureDimensions::Texture2dMultisample) &&
       (ty == TextureType::Regular ||
        ty == TextureType::Integral ||
        ty == TextureType::Unsigned)
    {
        (write!(dest, "
                /// Starts drawing on the texture.
                ///
                /// All the function calls to the framebuffer will draw on the texture instead
                /// of the screen.
                ///
                /// ## Low-level information
                ///
                /// The first time that this function is called, a FrameBuffer Object will be
                /// created and cached. The following calls to `as_surface` will load the existing
                /// FBO and re-use it. When the texture is destroyed, the FBO is destroyed too.
                ///
                #[inline]
                pub fn as_surface<'a>(&'a self) -> framebuffer::SimpleFrameBuffer<'a> {{
                    framebuffer::SimpleFrameBuffer::new(self.0.get_context(), self).unwrap()
                }}
            ")).unwrap();
    }

    // writing the `get_mipmap_levels` function
    (write!(dest, "
            /// Returns the number of mipmap levels of the texture.
            ///
            /// The minimum value is 1, since there is always a main texture.
            #[inline]
            pub fn get_mipmap_levels(&self) -> u32 {{
                self.0.get_mipmap_levels()
            }}
        ")).unwrap();

    // writing the `read` functions
    // TODO: implement for other types too
    if dimensions == TextureDimensions::Texture2d &&
       (ty == TextureType::Regular || ty == TextureType::Srgb || is_compressed)
    {
        (write!(dest, r#"
                /// Reads the content of the texture to RAM. This method may only read `U8U8U8U8`
                /// data, as it is the only format guaranteed to be supported across all OpenGL
                /// versions.
                ///
                /// You should avoid doing this at all cost during performance-critical
                /// operations (for example, while you're drawing).
                /// Use `read_to_pixel_buffer` instead.
                #[inline]
                pub fn read<T>(&self) -> T where T: Texture2dDataSink<(u8, u8, u8, u8)> {{
                    unsafe {{ self.unchecked_read() }}
                }}
            "#)).unwrap();

        (write!(dest, r#"
                /// Reads the content of the texture into a buffer in video memory. This method may
                /// only read `U8U8U8U8` data, as it is the only format guaranteed to be supported
                /// across all OpenGL versions.
                ///
                /// This operation copies the texture's data into a buffer in video memory
                /// (a pixel buffer). Contrary to the `read` function, this operation is
                /// done asynchronously and doesn't need a synchronization.
                #[inline]
                pub fn read_to_pixel_buffer(&self) -> PixelBuffer<(u8, u8, u8, u8)> {{
                    unsafe {{ self.unchecked_read_to_pixel_buffer() }}
                }}
            "#)).unwrap();

        (write!(dest, r#"
                /// Unsafely reads the content of the texture to RAM in the specified pixel format.
                /// It is possible that the current OpenGL context does not support the given
                /// format, in which case the returned data will be invalid.
                ///
                /// You should avoid doing this at all cost during performance-critical
                /// operations (for example, while you're drawing).
                /// Use `read_to_pixel_buffer` instead.
                #[inline]
                pub unsafe fn unchecked_read<T, P>(&self) -> T where T: Texture2dDataSink<P>, P: PixelValue {{
                    let rect = Rect {{ left: 0, bottom: 0, width: self.get_width(),
                                       height: self.get_height().unwrap_or(1) }};
                    self.0.main_level().first_layer().into_image(None).unwrap().raw_read(&rect)
                }}
            "#)).unwrap();

        (write!(dest, r#"
                /// Unsafely reads the content of the texture into a buffer in video memory. It is
                /// possible that the current OpenGL context does not support the given format, in
                /// which case the returned data will be invalid.
                ///
                /// This operation copies the texture's data into a buffer in video memory
                /// (a pixel buffer). Contrary to the `read` function, this operation is
                /// done asynchronously and doesn't need a synchronization.
                #[inline]
                pub unsafe fn unchecked_read_to_pixel_buffer<P>(&self) -> PixelBuffer<P> where P: PixelValue {{
                    let rect = Rect {{ left: 0, bottom: 0, width: self.get_width(),
                                       height: self.get_height().unwrap_or(1) }};
                    let pb = PixelBuffer::new_empty(self.0.get_context(),
                                                    rect.width as usize * rect.height as usize);
                    self.0.main_level().first_layer().into_image(None).unwrap()
                          .raw_read_to_pixel_buffer(&rect, &pb);
                    pb
                }}
            "#)).unwrap();
    }

    // writing the `read_compressed_data` function
    if is_compressed && !dimensions.is_array() {
        (write!(dest, r#"
                /// Reads the content of the texture to RAM without decompressing it before.
                ///
                /// You should avoid doing this at all cost during performance-critical
                /// operations (for example, while you're drawing).
                ///
                /// Returns the compressed format of the texture and the compressed data, gives
                /// `None` when the internal compression format is generic or unknown.
                #[inline]
                pub fn read_compressed_data(&self) -> Option<({format}, Vec<u8>)> {{
                    self.main_level().read_compressed_data()
                }}
            "#, format = relevant_format)).unwrap();
    }


    // writing the `write` function
    // TODO: implement for other types too
    if dimensions == TextureDimensions::Texture2d &&
            (ty == TextureType::Regular || ty == TextureType::Srgb || is_compressed)
    {
        let compressed_restrictions = if is_compressed {
            r#" ///
                /// Calling this for compressed textures will result in a panic of type INVALID_OPERATION
                /// if `Rect::bottom` or `Rect::width` is not equal to 0 (border). In addition, the contents
                /// of any texel outside the region modified by such a call are undefined. These
                /// restrictions may be relaxed for specific compressed internal formats whose images
                /// are easily edited.
            "#
        } else {
            ""
        };

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
                {compressed_restrictions}
                #[inline]
                pub fn write<'a, T>(&self, rect: Rect, data: T) where T: {data_source_trait}<'a> {{
                    self.main_level().write(rect, data)
                }}
            "#, data_source_trait = data_source_trait,
                compressed_restrictions = compressed_restrictions)).unwrap();
    }

    // writing the `write_compressed_data` function
    // TODO: implement for other types too
    if dimensions == TextureDimensions::Texture2d && is_compressed
    {
        (write!(dest, r#"
                /// Uploads some data in the texture by using a compressed format as input.
                ///
                /// Note that this may cause a synchronization if you use the texture right before
                /// or right after this call. Prefer creating a whole new texture if you change a
                /// huge part of it.
                ///
                /// ## Panic
                ///
                /// Panics if the the dimensions of `data` don't match the `Rect`.
                ///
                /// Calling this will result in a panic of type INVALID_OPERATION error if `Rect::width`
                /// or `Rect::height` is not equal to 0 (border), or if the written dimensions do not match
                /// the original texture dimensions. The contents of any texel outside the region modified
                /// by the call are undefined. These restrictions may be relaxed for specific compressed
                /// internal formats whose images are easily edited.
                #[inline]
                pub fn write_compressed_data(&self, rect: Rect, data: &[u8],
                                             width: u32, height: u32, format: {format})
                                             -> Result<(), ()>
                {{
                    // FIXME is having width and height as parameter redundant as rect kinda of
                    // already provides them?
                    self.main_level().write_compressed_data(rect, data, width, height, format)
                }}
            "#, format = relevant_format)).unwrap();
    }

    // `resident_if_supported`
    (write!(dest, r#"
            /// Turns the texture into a `ResidentTexture`.
            ///
            /// This allows you to use the texture in a much more efficient way by storing
            /// a "reference to it" in a buffer (actually not a reference but a raw pointer).
            ///
            /// See the documentation of `ResidentTexture` for more infos.
            #[inline]
            pub fn resident(self) -> Result<ResidentTexture, BindlessTexturesNotSupportedError> {{
                ResidentTexture::new(self.0)
            }}
        "#)).unwrap();

    // writing the layer & mipmap access functions
    if dimensions.is_array() {
        (write!(dest, r#"
                /// Access the first layer of this texture.
                #[inline]
                pub fn first_layer(&self) -> {name}Layer {{
                    self.layer(0).unwrap()
                }}

                /// Access a single layer of this texture.
                #[inline]
                pub fn layer(&self, layer: u32) -> Option<{name}Layer> {{
                    self.0.layer(layer).map(|l| {name}Layer(l, self))
                }}
            "#, name = name)).unwrap();
    }

    (write!(dest, r#"
            /// Access a single mipmap level of this texture.
            #[inline]
            pub fn mipmap(&self, level: u32) -> Option<{name}Mipmap> {{
                self.0.mipmap(level).map(|m| {name}Mipmap(m, self))
            }}

            /// Access the main mipmap level of this texture.
            #[inline]
            pub fn main_level(&self) -> {name}Mipmap {{
                self.mipmap(0).unwrap()
            }}
        "#, name = name)).unwrap();

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
                pub struct {name}Layer<'t>(TextureAnyLayer<'t>, &'t {name});
            "#, name = name)).unwrap();

        // opening `impl Layer` block
        (writeln!(dest, "impl<'t> {}Layer<'t> {{", name)).unwrap();

        // dimensions getters
        write_dimensions_getters(dest, dimensions, "(self.1).0", false);

        // writing the `get_layer` and `get_texture` functions
        (write!(dest, "
                /// Returns the corresponding texture.
                #[inline]
                pub fn get_texture(&self) -> &'t {name} {{
                    &self.1
                }}

                /// Returns the layer index.
                #[inline]
                pub fn get_layer(&self) -> u32 {{
                    self.0.get_layer()
                }}
            ", name = name)).unwrap();

        // writing the `get_mipmap_levels` function
        (write!(dest, "
                /// Returns the number of mipmap levels of the texture.
                ///
                /// The minimum value is 1, since there is always a main texture.
                #[inline]
                pub fn get_mipmap_levels(&self) -> u32 {{
                    self.0.get_texture().get_mipmap_levels()
                }}
            ")).unwrap();

        // writing the layer & mipmap access functions
        (write!(dest, r#"
            /// Access a single mipmap level of this layer.
            #[inline]
            pub fn mipmap(&self, level: u32) -> Option<{name}LayerMipmap<'t>> {{
                self.0.mipmap(level).map(|m| {name}LayerMipmap(m, self.1))
            }}

            /// Access the main mipmap level of this layer.
            #[inline]
            pub fn main_level(&self) -> {name}LayerMipmap<'t> {{
                self.mipmap(0).unwrap()
            }}
        "#, name = name)).unwrap();

        // closing `impl Layer` block
        (writeln!(dest, "}}")).unwrap();
    }

    // the `Mipmap` struct
    {
        // writing the struct
        (write!(dest, r#"
                /// Represents a single mipmap level of a `{name}`.
                ///
                /// Can be obtained by calling `{name}::mipmap()`, `{name}::main_level()`,
                /// `{name}Layer::mipmap()` or `{name}Layer::main_level()`.
                #[derive(Copy, Clone)]
                pub struct {name}Mipmap<'t>(TextureAnyMipmap<'t>, &'t {name});

                impl<'a> ::std::ops::Deref for {name}Mipmap<'a> {{
                    type Target = TextureAnyMipmap<'a>;

                    #[inline]
                    fn deref(&self) -> &TextureAnyMipmap<'a> {{
                        &self.0
                    }}
                }}
            "#, name = name)).unwrap();

        // opening `impl Mipmap` block
        (writeln!(dest, "impl<'t> {}Mipmap<'t> {{", name)).unwrap();

        // dimensions getters
        write_dimensions_getters(dest, dimensions, "self.0", true);

        // writing the `write` function for mipmaps.
        // TODO: implement for other types too
        if dimensions == TextureDimensions::Texture2d &&
                (ty == TextureType::Regular || ty == TextureType::Srgb || is_compressed)
        {
            let compressed_restrictions = if is_compressed {
                r#" ///
                    /// Calling this for compressed textures will result in a panic of type INVALID_OPERATION
                    /// if `Rect::bottom` or `Rect::width` is not equal to 0 (border). In addition, the contents
                    /// of any texel outside the region modified by such a call are undefined. These
                    /// restrictions may be relaxed for specific compressed internal formats whose images
                    /// are easily edited.
                "#
            } else {
                ""
            };

            (write!(dest, r#"
                    /// Uploads some data in the texture level.
                    ///
                    /// Note that this may cause a synchronization if you use the texture right before
                    /// or right after this call.
                    ///
                    /// ## Panic
                    ///
                    /// Panics if the the dimensions of `data` don't match the `Rect`.
                    {compressed_restrictions}
                    pub fn write<'a, T>(&self, rect: Rect, data: T) where T: {data_source_trait}<'a> {{
                        let RawImage2d {{ data, width, height, format: client_format }} =
                                                data.into_raw();

                        assert_eq!(width, rect.width);
                        assert_eq!(height, rect.height);

                        let client_format = ClientFormatAny::ClientFormat(client_format);

                        self.0.upload_texture(rect.left, rect.bottom, 0, (client_format, data),
                                              width, Some(height), None, true).unwrap()
                    }}
                "#, data_source_trait = data_source_trait,
                    compressed_restrictions = compressed_restrictions)).unwrap();
        }

        // writing the `write_compressed_data` function for mipmaps.
        // TODO: implement for other types too
        if dimensions == TextureDimensions::Texture2d && is_compressed
        {
            (write!(dest, r#"
                    /// Uploads some data in the texture level by using a compressed format as input.
                    ///
                    /// Note that this may cause a synchronization if you use the texture right before
                    /// or right after this call.
                    ///
                    /// ## Panic
                    ///
                    /// Panics if the the dimensions of `data` don't match the `Rect`.
                    ///
                    /// Calling this will result in a panic of type INVALID_OPERATION error if `Rect::width`
                    /// or `Rect::height` is not equal to 0 (border), or if the written dimensions do not match
                    /// the original texture dimensions. The contents of any texel outside the region modified
                    /// by the call are undefined. These restrictions may be relaxed for specific compressed
                    /// internal formats whose images are easily edited.
                    pub fn write_compressed_data(&self, rect: Rect, data: &[u8],
                                                 width: u32, height: u32, format: {format})
                                                 -> Result<(), ()>
                    {{
                        // FIXME is having width and height as parameter redundant as rect kinda of
                        // already provides them?

                        assert_eq!(width, rect.width);
                        assert_eq!(height, rect.height);

                        let data = Cow::Borrowed(data.as_ref());
                        let client_format = {client_format_any}(format);

                        self.0.upload_texture(rect.left, rect.bottom, 0, (client_format, data),
                                              width, Some(height), None, false)
                    }}
                "#, format = relevant_format, client_format_any = client_format_any_ty)).unwrap();
        }

        // writing the `read_compressed_data` function for mipmaps
        if is_compressed && !dimensions.is_array() {
            (write!(dest, r#"
                    /// Reads the content of the texture level to RAM without decompressing it before.
                    ///
                    /// You should avoid doing this at all cost during performance-critical
                    /// operations (for example, while you're drawing).
                    ///
                    /// Returns the compressed format of the texture and the compressed data, gives
                    /// `None` when the internal compression format is generic or unknown.
                    #[inline]
                    pub fn read_compressed_data(&self) -> Option<({format}, Vec<u8>)> {{
                        match self.0.download_compressed_data() {{
                            Some(({client_format_any}(format), buf)) => Some((format, buf)),
                            None => None,
                            _ => unreachable!(),
                        }}
                    }}
                "#, format = relevant_format, client_format_any = client_format_any_ty)).unwrap();
        }

        // writing the `get_level` and `get_texture` functions
        (write!(dest, "
                /// Returns the corresponding texture.
                #[inline]
                pub fn get_texture(&self) -> &'t {name} {{
                    self.1
                }}

                /// Returns the texture level.
                #[inline]
                pub fn get_level(&self) -> u32 {{
                    self.0.get_level()
                }}
        ", name = name)).unwrap();

        if dimensions.is_array() {
            (write!(dest, "
                    /// Access the first layer of this texture.
                    #[inline]
                    pub fn first_layer(&self) -> {name}LayerMipmap<'t> {{
                        self.layer(0).unwrap()
                    }}

                    /// Access a single layer of this texture.
                    #[inline]
                    pub fn layer(&self, layer: u32) -> Option<{name}LayerMipmap<'t>> {{
                        self.0.layer(layer).map(|l| {name}LayerMipmap(l, self.1))
                    }}
                ", name = name)).unwrap();
        }

        if !dimensions.is_array() && dimensions.is_cube() {
            writeln!(dest,
                "/// Provides an object representing a single layer of this cubemap.
                pub fn image(&self, layer: CubeLayer) -> {name}Image<'t> {{
                    {name}Image(self.0.first_layer().into_image(Some(layer)).unwrap(), self.1)
                }}", name = name).unwrap();
        }

        // closing `impl Mipmap` block
        (writeln!(dest, "}}")).unwrap();

        if !dimensions.is_array() && !dimensions.is_cube() {
            // into raw image
            (writeln!(dest, "impl<'t> Into<TextureAnyImage<'t>> for {name}Mipmap<'t> {{
                                fn into(self) -> TextureAnyImage<'t> {{
                                    self.0.first_layer().into_image(None).unwrap()
                                }}
                             }}", name = name)).unwrap();
        }
    }

    // the `LayerMipmap` struct
    if dimensions.is_array() {
        // writing the struct
        (write!(dest, r#"
                /// Represents a single layer of a mipmap level of a `{name}`.
                #[derive(Copy, Clone)]
                pub struct {name}LayerMipmap<'t>(TextureAnyLayerMipmap<'t>, &'t {name});
            "#, name = name)).unwrap();

        // opening `impl LayerMipmap` block
        (writeln!(dest, "impl<'t> {}LayerMipmap<'t> {{", name)).unwrap();

        // dimensions getters
        write_dimensions_getters(dest, dimensions, "self.0", false);

        // to the image struct
        if dimensions.is_cube() {
            writeln!(dest,
                "/// Provides an object representing a single layer of this cubemap.
                pub fn image(&self, layer: CubeLayer) -> {name}Image<'t> {{
                    {name}Image(self.0.into_image(Some(layer)).unwrap(), self.1)
                }}", name = name).unwrap();
        }

        // closing `impl LayerMipmap` block
        (writeln!(dest, "}}")).unwrap();

        // attachment traits
        if !dimensions.is_cube() {
            // into raw image
            (writeln!(dest, "impl<'t> Into<TextureAnyImage<'t>> for {name}LayerMipmap<'t> {{
                                fn into(self) -> TextureAnyImage<'t> {{
                                    self.0.into_image(None).unwrap()
                                }}
                             }}", name = name)).unwrap();
        }
    }

    // the `Image` struct, only for cubemaps
    if dimensions.is_cube() {
        // writing the struct
        (write!(dest, r#"
                /// Represents a single image of a mipmap level of a layer of `{name}`.
                #[derive(Copy, Clone)]
                pub struct {name}Image<'t>(TextureAnyImage<'t>, &'t {name});
            "#, name = name)).unwrap();

        // opening `impl Image` block
        (writeln!(dest, "impl<'t> {}Image<'t> {{", name)).unwrap();

        // dimensions getters
        write_dimensions_getters(dest, dimensions, "self.0", false);

        // closing `impl Image` block
        (writeln!(dest, "}}")).unwrap();

        // into raw image
        (writeln!(dest, "impl<'t> Into<TextureAnyImage<'t>> for {name}Image<'t> {{
                            fn into(self) -> TextureAnyImage<'t> {{
                                self.0
                            }}
                         }}", name = name)).unwrap();
    }

    // implement the attachments traits
    {
        let attachment_type = if dimensions.is_cube() {
            format!("{}Image", name)
        } else if dimensions.is_array() {
            format!("{}LayerMipmap", name)
        } else {
            format!("{}Mipmap", name)
        };

        match ty {
            TextureType::Regular | TextureType::Srgb | TextureType::Integral | TextureType::Unsigned => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToColorAttachment<'t> for {ty}<'t> {{
                            #[inline]
                            fn to_color_attachment(self) -> ::framebuffer::ColorAttachment<'t> {{
                                ::framebuffer::ColorAttachment::Texture(self.into())
                            }}
                        }}
                    ", ty = attachment_type)).unwrap();
            },
            TextureType::Depth => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToDepthAttachment<'t> for {ty}<'t> {{
                            #[inline]
                            fn to_depth_attachment(self) -> ::framebuffer::DepthAttachment<'t> {{
                                ::framebuffer::DepthAttachment::Texture(self.into())
                            }}
                        }}
                    ", ty = attachment_type)).unwrap();
            },
            TextureType::Stencil => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToStencilAttachment<'t> for {ty}<'t> {{
                            #[inline]
                            fn to_stencil_attachment(self) -> ::framebuffer::StencilAttachment<'t> {{
                                ::framebuffer::StencilAttachment::Texture(self.into())
                            }}
                        }}
                    ", ty = attachment_type)).unwrap();
            },
            TextureType::DepthStencil => {
                (writeln!(dest, "
                        impl<'t> ::framebuffer::ToDepthStencilAttachment<'t> for {ty}<'t> {{
                            #[inline]
                            fn to_depth_stencil_attachment(self) -> ::framebuffer::DepthStencilAttachment<'t> {{
                                ::framebuffer::DepthStencilAttachment::Texture(self.into())
                            }}
                        }}
                    ", ty = attachment_type)).unwrap();
            },
            _ => ()
        }
    }

    // closing `mod module {`
    writeln!(dest, "}}").unwrap();
}

fn write_dimensions_getters<W: Write>(dest: &mut W, dimensions: TextureDimensions,
                                      accessor: &str, write_array_size: bool)
{
    writeln!(dest, r#"
        /// Returns the width of that image.
        #[inline]
        pub fn width(&self) -> u32 {{
            {}.get_width()
        }}
    "#, accessor).unwrap();

    match dimensions {
        TextureDimensions::Texture2d | TextureDimensions::Texture2dMultisample |
        TextureDimensions::Texture3d | TextureDimensions::Texture2dArray |
        TextureDimensions::Texture2dMultisampleArray | TextureDimensions::Cubemap |
        TextureDimensions::CubemapArray => {
            writeln!(dest, r#"
                /// Returns the height of that image.
                #[inline]
                pub fn height(&self) -> u32 {{
                    {}.get_height().unwrap()
                }}
            "#, accessor).unwrap();
        },
        _ => ()
    };

    match dimensions {
        TextureDimensions::Texture3d => {
            writeln!(dest, r#"
                /// Returns the depth of that image.
                #[inline]
                pub fn depth(&self) -> u32 {{
                    {}.get_depth().unwrap()
                }}
            "#, accessor).unwrap();
        },
        _ => ()
    };

    if write_array_size {
        match dimensions {
            TextureDimensions::Texture2dArray | TextureDimensions::Texture2dMultisampleArray |
            TextureDimensions::CubemapArray => {
                writeln!(dest, r#"
                    /// Returns the number of array layers.
                    #[inline]
                    pub fn array_size(&self) -> u32 {{
                        {}.get_array_size().unwrap()
                    }}
                "#, accessor).unwrap();
            },
            _ => ()
        }
    };

    match dimensions {
        TextureDimensions::Texture2dMultisample | TextureDimensions::Texture2dMultisampleArray => {
            writeln!(dest, r#"
                /// Returns the number of samples of that image.
                #[inline]
                pub fn samples(&self) -> u32 {{
                    {}.get_samples().unwrap()
                }}
            "#, accessor).unwrap();
        },
        _ => ()
    };

    match dimensions {
        TextureDimensions::Texture2d | TextureDimensions::Texture2dMultisample |
        TextureDimensions::Texture2dArray | TextureDimensions::Texture2dMultisampleArray => {
            writeln!(dest, r#"
                /// Returns the width and height of that image.
                #[inline]
                pub fn dimensions(&self) -> (u32, u32) {{
                    (self.width(), self.height())
                }}
            "#).unwrap();
        },
        TextureDimensions::Texture3d => {
            writeln!(dest, r#"
                /// Returns the width, height and depth of that image.
                #[inline]
                pub fn dimensions(&self) -> (u32, u32, u32) {{
                    (self.width(), self.height(), self.depth())
                }}
            "#).unwrap();
        },
        TextureDimensions::Cubemap | TextureDimensions::CubemapArray => {
            writeln!(dest, r#"
                /// Returns the dimension of that image.
                #[inline]
                pub fn dimensions(&self) -> u32 {{
                    self.width()
                }}
            "#).unwrap();
        },
        _ => ()
    };
}
