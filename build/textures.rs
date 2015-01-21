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
    Texture3d,
    Texture1dArray,
    Texture2dArray,
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
            TextureDimensions::Texture3d => "Texture3d",
            TextureDimensions::Texture1dArray => "Texture1dArray",
            TextureDimensions::Texture2dArray => "Texture2dArray",
        };

        format!("{}{}", prefix, suffix)
    };

    // writing the struct with doc-comment
    (write!(dest, "/// ")).unwrap();
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture2d |
        TextureDimensions::Texture3d => "A ",
        TextureDimensions::Texture1dArray | TextureDimensions::Texture2dArray => "An array of ",
    })).unwrap();
    if ty == TextureType::Compressed {
        (write!(dest, "compressed ")).unwrap();
    }
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "one-dimensional ",
        TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => "two-dimensional ",
        TextureDimensions::Texture3d => "three-dimensional ",
    })).unwrap();
    (write!(dest, "{}", match dimensions {
        TextureDimensions::Texture1d | TextureDimensions::Texture2d |
        TextureDimensions::Texture3d => "texture ",
        TextureDimensions::Texture1dArray | TextureDimensions::Texture2dArray => "textures ",
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
                    fn get_id(&self) -> gl::types::GLuint {{
                        self.0.get_id()
                    }}
                }}
            ", name)).unwrap();

    // `Show` trait impl
    (writeln!(dest, "
                impl ::std::fmt::Show for {} {{
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error>
                    {{
                        self.0.fmt(f)
                    }}
                }}
            ", name)).unwrap();

    // `UniformValue` trait impl
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

    // `ToXXXAttachment` trait impl
    if dimensions == TextureDimensions::Texture2d {
        match ty {
            TextureType::Regular => {
                (writeln!(dest, "
                        impl ::framebuffer::ToColorAttachment for {} {{
                            fn to_color_attachment(&self) -> ::framebuffer::ColorAttachment {{
                                ::framebuffer::ColorAttachment::Texture2d(self)
                            }}
                        }}
                    ", name)).unwrap();
            },
            TextureType::Depth => {
                (writeln!(dest, "
                        impl ::framebuffer::ToDepthAttachment for {} {{
                            fn to_depth_attachment(&self) -> ::framebuffer::DepthAttachment {{
                                ::framebuffer::DepthAttachment::Texture2d(self)
                            }}
                        }}
                    ", name)).unwrap();
            },
            TextureType::Stencil => {
                (writeln!(dest, "
                        impl ::framebuffer::ToStencilAttachment for {} {{
                            fn to_stencil_attachment(&self) -> ::framebuffer::StencilAttachment {{
                                ::framebuffer::StencilAttachment::Texture2d(self)
                            }}
                        }}
                    ", name)).unwrap();
            },
            TextureType::DepthStencil => {
                (writeln!(dest, "
                        impl ::framebuffer::ToDepthStencilAttachment for {} {{
                            fn to_depth_stencil_attachment(&self) -> ::framebuffer::DepthStencilAttachment {{
                                ::framebuffer::DepthStencilAttachment::Texture2d(self)
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
    {
        let data_type = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "Texture1dData",
            TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => "Texture2dData",
            TextureDimensions::Texture3d => "Texture3dData",
        };

        let param = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture2d |
            TextureDimensions::Texture3d => "T",

            TextureDimensions::Texture1dArray |
            TextureDimensions::Texture2dArray => "Vec<T>",
        };

        (writeln!(dest, "
                /// Builds a new texture by uploading data.
                ///
                /// This function will automatically generate all mipmaps of the texture.
                pub fn new<T>(display: &::Display, data: {param})
                              -> {name} where T: {data_type}
                {{
            ", data_type = data_type, param = param, name = name)).unwrap();


        // writing the `let format = ...` line
        match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => {
                (writeln!(dest, "let format = <T as Texture1dData>::get_format();")).unwrap();
            },
            TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => {
                (writeln!(dest, "let format = <T as Texture2dData>::get_format();")).unwrap();
            },
            TextureDimensions::Texture3d => {
                (writeln!(dest, "let format = <T as Texture3dData>::get_format();")).unwrap();
            },
        }
        match ty {
            TextureType::Compressed => {
                (write!(dest, "let format = format.to_default_compressed_format();")).unwrap();
            },
            TextureType::Regular | TextureType::Integral | TextureType::Unsigned => {
                (write!(dest, "let format = format.to_default_float_format();")).unwrap();
            },
            TextureType::Depth => {
                (write!(dest, "let format = DepthFormat::I24.to_glenum();")).unwrap();
            },
            TextureType::Stencil => {
                (write!(dest, "let format = StencilFormat::I8.to_glenum();")).unwrap();
            },
            TextureType::DepthStencil => {
                (write!(dest, "let format = DepthStencilFormat::I24I8.to_glenum();")).unwrap();
            },
        };

        // writing the `let (client_format, client_type) = ...` line
        match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => {
                (writeln!(dest, "let client_format = <T as Texture1dData>::get_format();")).unwrap();
            },
            TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => {
                (writeln!(dest, "let client_format = <T as Texture2dData>::get_format();")).unwrap();
            },
            TextureDimensions::Texture3d => {
                (writeln!(dest, "let client_format = <T as Texture3dData>::get_format();")).unwrap();
            },
        }
        (write!(dest, "let (client_format, client_type) = ")).unwrap();
        match ty {
            TextureType::Compressed | TextureType::Regular | TextureType::Depth => {
                (write!(dest, "client_format.to_gl_enum()")).unwrap();
            },
            TextureType::Integral | TextureType::Stencil => {
                (write!(dest, "client_format.to_gl_enum_int().expect(\"Client format must \
                               have an integral format\")")).unwrap();
            },
            TextureType::Unsigned => {
                (write!(dest, "client_format.to_gl_enum_uint().expect(\"Client format must \
                               have an integral format\")")).unwrap();
            },
            TextureType::DepthStencil => {
                (write!(dest, "unimplemented!()")).unwrap();
            },
        };
        (writeln!(dest, ";")).unwrap();


        match dimensions {
            TextureDimensions::Texture1d => (write!(dest, "
                    let data = data.into_vec();
                    let width = data.len() as u32;
                ")).unwrap(),

            TextureDimensions::Texture2d => (write!(dest, "
                    let (width, height) = data.get_dimensions();
                    let width = width as u32; let height = height as u32;
                    let data = data.into_vec();
                ")).unwrap(),

            TextureDimensions::Texture3d => (write!(dest, "
                    let (width, height, depth) = data.get_dimensions();
                    let width = width as u32; let height = height as u32; let depth = depth as u32;
                    let data = data.into_vec();
                ")).unwrap(),

            TextureDimensions::Texture1dArray => (write!(dest, "
                    let array_size = data.len() as u32;
                    let mut width = 0;
                    let data = data.into_iter().flat_map(|t| {{
                        let d = t.into_vec(); width = d.len(); d.into_iter()
                    }}).collect();
                    let width = width as u32;
                ")).unwrap(),   // TODO: panic if dimensions are inconsistent

            TextureDimensions::Texture2dArray => (write!(dest, "
                    let array_size = data.len() as u32;
                    let mut dimensions = (0, 0);
                    let data = data.into_iter().flat_map(|t| {{
                        dimensions = t.get_dimensions(); t.into_vec().into_iter()
                    }}).collect();
                    let (width, height) = dimensions;
                    let width = width as u32; let height = height as u32;
                ")).unwrap(),   // TODO: panic if dimensions are inconsistent
        }
        // writing the constructor
        (write!(dest, "{}(TextureImplementation::new(display, format, Some(data), \
                       client_format, client_type, ", name)).unwrap();
        match dimensions {
            TextureDimensions::Texture1d => (write!(dest, "width, None, None, None")).unwrap(),
            TextureDimensions::Texture2d => (write!(dest, "width, Some(height), None, None")).unwrap(),
            TextureDimensions::Texture3d => (write!(dest, "width, Some(height), Some(depth), None")).unwrap(),
            TextureDimensions::Texture1dArray => (write!(dest, "width, None, None, Some(array_size)")).unwrap(),
            TextureDimensions::Texture2dArray => (write!(dest, "width, Some(height), None, Some(array_size)")).unwrap(),
        }
        (writeln!(dest, "))")).unwrap();

        // end of "new" function block
        (writeln!(dest, "}}")).unwrap();
    }

    // writing the `new_empty` function
    if ty != TextureType::Compressed {
        let format = match ty {
            TextureType::Regular => "UncompressedFloatFormat",
            TextureType::Compressed => "CompressedFormat",
            TextureType::Integral => "UncompressedIntFormat",
            TextureType::Unsigned => "UncompressedUintFormat",
            TextureType::Depth => "DepthFormat",
            TextureType::Stencil => "StencilFormat",
            TextureType::DepthStencil => "DepthStencilFormat",
        };

        let dim_params = match dimensions {
            TextureDimensions::Texture1d => "width: u32",
            TextureDimensions::Texture2d => "width: u32, height: u32",
            TextureDimensions::Texture3d => "width: u32, height: u32, depth: u32",
            TextureDimensions::Texture1dArray => "width: u32, array_size: u32",
            TextureDimensions::Texture2dArray => "width: u32, height: u32, array_size: u32",
        };

        // opening function
        (writeln!(dest, "
                /// Creates an empty texture.
                ///
                /// The texture will contain undefined data.
                pub fn new_empty(display: &::Display, format: {format}, {dim_params}) -> {name} {{
                    let format = format.to_glenum();
            ", format = format, dim_params = dim_params, name = name)).unwrap();

        // writing the constructor
        (write!(dest, "{}(TextureImplementation::new::<u8>(display, format, None, \
                       gl::RGBA, gl::UNSIGNED_BYTE, ", name)).unwrap();
        match dimensions {
            TextureDimensions::Texture1d => (write!(dest, "width, None, None, None")).unwrap(),
            TextureDimensions::Texture2d => (write!(dest, "width, Some(height), None, None")).unwrap(),
            TextureDimensions::Texture3d => (write!(dest, "width, Some(height), Some(depth), None")).unwrap(),
            TextureDimensions::Texture1dArray => (write!(dest, "width, None, None, Some(array_size)")).unwrap(),
            TextureDimensions::Texture2dArray => (write!(dest, "width, Some(height), None, Some(array_size)")).unwrap(),
        }
        (writeln!(dest, "))")).unwrap();

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

    // writing the `read` functions
    // TODO: implement for other types too
    if dimensions == TextureDimensions::Texture2d &&
       (ty == TextureType::Regular || ty == TextureType::Compressed)
    {
        /*let data_type = match dimensions {
            TextureDimensions::Texture1d | TextureDimensions::Texture1dArray => "Texture1dData",
            TextureDimensions::Texture2d | TextureDimensions::Texture2dArray => "Texture2dData",
            TextureDimensions::Texture3d => "Texture3dData",
        };*/

        (write!(dest, r#"
                /// Reads the content of the texture to RAM.
                ///
                /// You should avoid doing this at all cost during performance-critical
                /// operations (for example, while you're drawing).
                /// Use `read_to_pixel_buffer` instead.
                pub fn read<P, T>(&self) -> T where T: Texture2dData<Data = P>, P: PixelValue + Clone {{    // TODO: remove Clone
                    self.0.read(0)
                }}
            "#)).unwrap();

        (write!(dest, r#"
                /// Reads the content of the texture into a buffer in video memory.
                ///
                /// This operation copies the texture's data into a buffer in video memory
                /// (a pixel buffer). Contrary to the `read` function, this operation is
                /// done asynchronously and doesn't need a synchronization.
                pub fn read_to_pixel_buffer<P, T>(&self) -> PixelBuffer<T> where T: Texture2dData<Data = P>, P: PixelValue + Clone {{    // TODO: remove Clone
                    self.0.read_to_pixel_buffer(0)
                }}
            "#)).unwrap();
    }

    // closing `impl Texture` block
    (writeln!(dest, "}}")).unwrap();
}
