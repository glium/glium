# Change Log

## Version 0.25.1 (2019-07-08)

- Silenced deprecation warning when invoking `implement_vertex`.

## Version 0.25.0 (2019-05-24)

- Updated glutin to version 0.21.

## Version 0.24.0 (2019-04-08)

 - Updated glutin to version 0.20. See the glutin release notes [here](https://github.com/tomaka/glutin/blob/master/CHANGELOG.md#version-0200-2019-03-09).
 - Depth comparison and shadow mapping ([#1679](https://github.com/glium/glium/pull/1679)).

## Version 0.23.0 (2018-12-05)

 - Updated glutin to version 0.19. See the glutin release notes [here](https://github.com/tomaka/glutin/blob/master/CHANGELOG.md#version-0190-2018-11-09).

## Version 0.22.0 (2018-07-02)

 - Updated glutin to version 0.17.
 - Updated glutin to version 0.16. Added 'icon_loading' feature.

## Version 0.21.0 (2018-04-11)

 - Updated glutin to version 0.14. Fixes handling of HiDPI on macOS.
 - Updated gl_generator to version 0.9.

## Version 0.20.0 (2018-01-22)

 - Updated glutin to version 0.12.
 - Updated smallvec from version 0.4 to 0.6.
 - Updated misc internal dependencies and dev-dependencies (lazy_static, cgmath, rand, image, gl_generator).
 - Replaced the `IntoVerticesSource` trait with `Into<VerticesSource>`.
 - Fixed [rendering bug](https://github.com/glium/glium/issues/1657) on high-DPI screens.
 - Added support for clipping via `gl_ClipDistance`.
 - Enabled [depth-stencil FBO attachments](https://github.com/glium/glium/issues/253).

## Version 0.19.0 (2017-12-11)

 - Updated glutin to version 0.11. Notably includes the [winit 0.9 update](https://github.com/tomaka/winit/blob/master/CHANGELOG.md#version-090-2017-12-01).
 - Updated gl_generator to 0.7 (internal dependency).

## Version 0.18.1 (2017-11-05)

 - Fixed links pointing to tomaka/glium instead of glium/glium
 - Various documentation updates, bugfixes, and dependency updates.

## Version 0.18.0 (2017-10-15)

 - Updated glutin to version 0.10.
 - Added support for is_buffer_supported usage on all buffer types.
 - Various documentation updates and bugfixes.

## Version 0.17.1 (2017-08-27)

 - Changed documentation to docs.rs.
 - Various bugfixes and updates to internal dependencies.

## Version 0.17.0 (2017-07-12)

 - Updated glutin to version 0.9.
 - Redesigned API around EventsLoop to match updated winit design.
 - Added support for vector normalization.
 - Various bugfixes.

## Version 0.16.0 (2017-01-07)

 - Added asynchronous screenshot example.
 - Updated tutorials to compile on Mac OS X.
 - Various tutorial documentation updates.
 - Fix buffer reads which could fail being safe.
 - Various bugfixes.

## Version 0.15.0 (2016-07-03)

 - Updated glutin to version 0.6.1
 - Various internal dependency updates.

## Version 0.14.0 (2016-04-11)

 - Updated glutin to version 0.5.
 - Various bugfixes.

## Version 0.13.5 (2016-02-04)

 - Fixed integer textures using a forbidden filtering by default.
 - Better error report in case of a uniform block layout mismatch.

## Version 0.13.4 (2016-01-21)

 - Added support for shader subroutines.
 - Added various functions to `Context` to retrieve information (like the content of `GL_VENDOR` or `GL_RENDERER` for example).
 - Added additional dimensions getters to the various texture types.

## Version 0.13.3 (2016-01-08)

 - Added constructors to `SimpleFrameBuffer` for depth-only, depth-stencil-only, stencil-only and depth & stencil framebuffers.

## Version 0.13.2 (2015-12-21)

 - Fixed compilation on ARM platform.
 - Added a hack to make screenshots and video taken with the FRAPS software work.

## Version 0.13.1 (2015-12-15)

 - `raw_read_to_pixel_buffer` now accepts all pixel types.

## Version 0.13.0 (2015-12-12)

 - Removed native support for cgmath and nalgebra in avoid to avoid dependency hell.
 - Reworked the layer-related, level-related functions and `into_image` from texture types where they are not relevant.
 - The `Backend::get_proc_address` function now takes a `*const std::os::raw::c_void` instead of a `*const ()`.

## Version 0.12.4 (2015-12-12)

 - Added small hack to make glium work with WebGL.
 - Fixed blend color state not being always updated.

## Version 0.12.3 (2015-12-04)

 - Added `raw_clear_buffer` to `TextureAnyImage`, which allows you to clear the content of any texture.
 - Added `TextureAny::from_id` to manipulate an unowned texture.

## Version 0.12.2 (2015-11-25)

 - Fixed a bug when matching frag output locations with framebuffer attachments.

## Version 0.12.1 (2015-11-25)

 - Added `MultiOutputFrameBuffer::with_stencil`, `with_depth_stencil` and `with_depth_and_stencil`.
 - Implemented the `ToColorAttachment` trait for unsigned and integral textures.
 - Implemented the `ToXAttachment` traits for the `XAttachment` types.
 - It is now possible to use another format than `(u8, u8, u8, u8)` with `raw_read` on a texture.

## Version 0.12.0 (2015-11-21)

 - Removed the "image" feature and native support for the `image`. You now have to perform conversions yourself.
 - The `Backend::get_proc_address` function now takes a `*const ()` instead of a `*const libc::c_void`.
 - Creating a `MultiOutputFrameBuffer` now requires passing an iterator instead of a `Vec`.
 - When checking a uniform block's layout, `[f32; 4]` now matches both `vec4` and `float[4]` instead of just `vec4` before. Same for all other arrays between 1 and 4 elements.
 - `UniformBlock` is now implemented for `[T; N]` where N is between 5 and 32, plus 64, 128, 256, 512, 1024 and 2048.
 - When checking a uniform block's layout, arrays (sized and unsized) now match structs with a single member that is an array.

## Version 0.11.1 (2015-11-10)

 - Fix broken compilation after libc 0.2.

## Version 0.11.0 (2015-11-04)

 - Updated the versions of glutin, image and cgmath.
 - Improved performance of stencil-related state changes.
 - Fixed `glBufferData` being called to invalidate a buffer created with `glBufferStorage`.
 - Changed use of `Range` in buffer slice to `RangeArgument`.

## Version 0.10.0 (2015-10-14)

 - Update glutin to 0.4, cgmath to 0.3 and nalgebra to 0.3.
 - Add the possibility to set the behavior of the debug callback in `Context::new`.
 - Add `build_glium_debug` and `build_glium_unchecked_debug` to set the behavior of the debug callback at initialization.
 - Rename `ClockWise` to `Clockwise`.
 - The `program!` macro now returns a `ProgramChooserCreationError` and no longer panics if no version is found.
 - Remove the `Texture` trait. All of its methods are already implemented on all texture types already.
 - Rename `UniformBlock::binding` to `id` and add `initial_binding`.

## Version 0.9.3 (2015-10-13)

 - Fixed an invalid enum error during initialization.

## Version 0.9.2 (2015-10-11)

 - Add `is_color_renderable` to the color image formats.
 - Fixed glium erroneously using a SSBO's and a uniform buffer's binding point instead of index.
 - Fixed `glInvalidateBuffer(Sub)Data` being called for persistent-mapped buffers.

## Version 0.9.1 (2015-09-30)

 - Add `ComputeShader::execute_indirect`.
 - Add support for 64bits integer uniforms.
 - `copy_to` now takes a `Into<BufferSlice>` instead of a `BufferSlice`.
 - Glium no longer calls `glBlendColor` if the blending algorithm doesn't use it.
 - Deprecated `DrawParameters::validate`.

## Version 0.9.0 (2015-09-15)

 - The blending, depth and stencil functions now use dedicated structs named `Blend`, `Depth` and `Stencil`.
 - `SimpleFrameBuffer` and `MultiOutputFrameBuffer`'s constructors now return a `Result`.
 - Creating a renderbuffer now returns a `Result`.
 - Add support for empty framebuffers with the `EmptyFrameBuffer` type.
 - Fix an OpenGL error when mapping a buffer with `map_read`.
 - Glium no longer panics in case of OpenGL error. It prints a message with backtrace on stdout instead.
 - Removed the `DrawParametersBuilder` struct.
 - Add `clear_color_srgb` and derivates. The `clear` method has an additional `color_srgb: bool` parameter.
 - Added some state changes when swapping buffers to adapt to the FRAPS software.
 - Removed `StencilTexture3d` are they are not supported by OpenGL.

## Version 0.8.7 (2015-08-27)

 - Fix a panic when creating a stencil renderbuffer or texture.
 - Add support for `f64` uniforms.

## Version 0.8.6 (2015-08-26)

 - Renamed `BufferView` to `Buffer`. `BufferView` still exists for backward compatibility.
 - Add a `copy_to` method to buffers.
 - Add support for the `GL_OES_element_index_uint` extension.
 - Fixed OpenGL ES 3.2 not working.
 - Add support for `IntVec{2|3|4}` and `UnsignedIntVec{2|3|4}` uniform types.
 - Add a work-around for Radeon drivers crashing with 32+ texture units.
 - Add support for `boolean` uniforms.
 - Add support for `BoolVec{2|3|4}` uniform types.

## Version 0.8.5 (2015-08-12)

 - Added support for cubemaps and cubemap arrays. Not all operations are available yet.
 - `DrawParameters` no longer implements `Copy`.
 - Add support for OpenGL ES 3.2 for geometry shaders, tessellation shaders, robustness, debug output, buffer textures, stencil textures, base vertex.
 - Add support for `GL_OES_geometry_shader`, `GL_OES_tessellation_shader` `GL_OES_draw_elements_base_vertex`, `GL_OES_draw_elements_base_vertex`.
 - Add support for specifying the primitive bounding box as an optimization hint to the backend.
 - Added various `is_texture_*_supported` functions in the `texture` module to check whether a texture type is supported.

## Version 0.8.4 (2015-08-10)

 - Added `#[inline]` attributes on many functions.
 - Added `slice_custom` and `slice_custom_mut` to buffers, which allow you to get slices over anything inside the buffer.
 - Added `Query::to_buffer_u32` to write the result of a query to a buffer.
 - Added `is_supported` functions for all the texture and renderbuffer formats.
 - Added missing `I16I16I16I16` and `U16U16U16` float formats. Renamed `U3U32U`to `U3U3U2`.
 - Various fixes and improvements when determining whether a format is supported.
 - Changed the `framebuffer::To*Attachment` traits with a lifetime parameter, and to take by value instead of by reference.
 - The `framebuffer::To*Attachment` traits are now implemented on `TextureLayerMipmap` structs.
 - Fixed `.layer()` function returning `None` when it shouldn't.
 - GLSL ES 3.2 is now recognized.

## Version 0.8.3 (2015-08-04)

 - Textures are now inside submodules (for example `Texture2d` is in `texture::texture2d::Texture2d`) and reexported from `texture`.
 - Added `Context::flush()` and `Context::finish()`. Deprecated `Context::synchronize`.
 - Removed `Sized` constraint for `Surface` that was preventing one from using `&Surface` or `&mut Surface`.
 - Added `TextureAnyMipmap::raw_upload_from_pixel_buffer`.
 - Moved the `pixel_buffer` module to `texture::pixel_buffer` (the old module still exists for backward compatibility).

## Version 0.8.2 (2015-07-29)

 - Added a `buffer_texture` module in `texture`.
 - Added `DrawParameters::depth_clamp` and `DrawError::DepthClampNotSupported`.

## Version 0.8.1 (2015-07-27)

 - Added `DrawParameters::ProvokingVertex` and `DrawError::ProvokingVertexNotSupported`.
 - Added `DrawError::RasterizerDiscardNotSupported`.
 - `DrawParametersBuilder` is now deprecated.
 - `ProgramCreationInput` now has a `outputs_srgb` member.
 - Fixed a bug with offsets of members in arrays.
 - Added support for the `GL_EXT_geometry_shader` OpenGL ES extension.

## Version 0.8.0 (2015-07-19)

 - Removed all Cargo features related to OpenGL compatibility.
 - Replaced all `new_if_supported` or `empty_if_supported` functions with `new` or `empty` and a proper error type.
 - All `new` and `empty` constructors now return an error if the operation is not supported.
 - `VertexBuffer` and `IndexBuffer` constructors now take a `&[T]`.
 - Updated glutin to version 0.3.
 - `BufferView`'s constructors now take a `BufferMode`.
 - Replaced `read` and `read_if_supported` with `read` that returns a `ReadError`.
 - Added `buffer::is_buffer_read_supported`.
 - Removed the deprecated `new_empty` function from textures.
 - Reworked `TextureCreationError` and removed `TextureMaybeSupportedCreationError`.
 - `new` and `dynamic` are now less specialized, which should fix some performance bottlenecks.
 - All buffer constructors now come in four variants: `new`, `dynamic`, `persistent`, `immutable`.
 - Add support for all missing vertex attributes.
 - Fixed stencil operations sometimes not working.

## Version 0.7.1 (2015-07-14)

 - Glium now automatically calls `glDraw*BaseVertex` if it is supported.
 - Added the `CapabilitiesSource` trait implemented automatically on all types that implement `Facade`.
 - Added `is_supported` to `IndexType`, `Index`, `PrimitiveType`, `Attribute` and `AttributeType`.
 - Added `ComputeShader::is_supported`, `program::is_geometry_shader_supported` and `program::is_tessellation_shader_supported`.
 - The `implement_buffer_content!` and `implement_block_layout!` macros can now take a struct with a lifetime parameter by passing `Foo<'a>` instead of `Foo`.

## Version 0.7.0 (2015-07-08)

 - Buffers now contain a single, possibly unsized, element. `BufferView<T>` is now `BufferView<[T]>`.
 - Creating an empty `BufferView<[T]>` now requires calling `empty_array` instead of `empty`.
 - `BufferView::write` now takes a `&T` instead of a `P where P: AsRef<[T]>`. This means that you can no longer pass `Vec`s by value for example.
 - Removed the mock methods from `UniformBuffer` as they are available through the `Deref` to `BufferView`.
 - The `UniformBlock` trait is now implemented on `DrawCommandNoIndices` and `DrawCommandIndices`.
 - Added the `implement_buffer_content!` macro to put on unsized struct so that you can put them inside buffers.
 - Added a `uniforms::LayoutMismatchError` as an additional field to `DrawError::UniformBlockLayoutMismatch`.
 - Changed `UniformBlock::matches` to return a `Result<(), UniformBlockLayoutMismatch>`.
 - Removed the `matches` function from the `AsUniformValue` trait.
 - Removed `TextureSurface` in favor of `SimpleFrameBuffer`.

## Version 0.6.7 (2015-07-06)

 - Added `DrawCommandsIndicesBuffer` for multidraw elements indirect.
 - Added `ResidentTexture`, `TextureHandle` and `Texture::resident_if_supported()` for bindless textures.
 - Buffers no longer require their content to be `Send` or `'static` (except for types with the `Any` suffix).
 - Renamed `texture::TextureType` to `Dimensions`.
 - Added `Context::get_opengl_version()` and deprecated `get_version()`.

## Version 0.6.6 (2015-07-03)

 - Buffers with persistent mapping now synchronize individual segments of the buffer instead of the buffer as a whole.
 - Dynamic sized arrays are now properly handled by the introspection system.
 - Added support for matrix shader attributes.
 - Fixed the value of `depth_write` being ignored if `depth_test` was `Overwrite`.

## Version 0.6.5 (2015-06-29)

 - Renamed and changed `UniformBlockMember` to `BlockLayout`.
 - Added `map_read()` and `map_write()` to buffers.
 - Various bugfixes related to queries.
 - Added a `implement_uniform_block!` macro for uniform buffers and SSBOs similar to `implement_vertex!`.
 - Added `UniformBuffer::empty` and `UniformBuffer::empty_if_supported`.
 - Fixed sRGB being disabled when clearing before drawing for the first time.

## Version 0.6.4

 - Fixed an OpenGL error if GL_ARB_robustness was present with OpenGL < 3.0.
 - Now using the `GL_APPLE_SYNC` extension if OpenGL ES 3.0 is not available.
 - Now using the `GL_EXT_buffer_storage` extension on OpenGL ES if it is available.
 - Slightly improved performances when using a dynamic buffer.
 - Added `GL_PROGRAM_POINT_SIZE` support to `Program`.

## Version 0.6.3

 - Fixed the OpenGL compatibility check for SSBO reflection.
 - Fixed a potential OpenGL error if uniform buffer objects are not supported.
 - Fixed a potential OpenGL error if transform feedback buffers are not supported.

## Version 0.6.2

 - Added `is_context_lost`, `is_robust` and `is_context_loss_possible` methods to `Context` (callable through the `Display` as well).
 - Fixed using a framebuffer with a depth or stencil attachment causing the wrong texture to be displayed.

## Version 0.6.1

 - Fixed mipmaps generation with sRGB textures.

## Version 0.6.0

 - `Frame::finish` now returns a `Result`. `Frame`'s destructor will panic if `finish` has not been called.
 - `with_compressed_data` and `with_compressed_data_if_supported` now have an additional parameter for mipmaps.
 - All the texture constructors that used to take a boolean as parameter for mipmaps now takes an enum.
 - `empty_with_format` and `empty_with_format_if_supported` are now allowed for compressed textures.
 - `write`, `write_compressed_data`, `write_compressed_data_if_supported` and `read_compressed_data` are now available for mipmap objects.
 - Removed the `is_closed()` function.
 - Removed the deprecated `render_buffer` module.
 - The `Backend::swap_buffers` function must now return a `Result`.

## Version 0.5.6

 - The panic at initialization in case of OpenGL error has been replaced by a warning.

## Version 0.5.5

 - Added `with_compressed_data`, `read_compressed_data` and `write_compressed_data` to compressed texture types.
 - Fixed a panic at initialization on OS/X.

## Version 0.5.4

 - Added the `LineLoop` primitive type.
 - Added the `smooth` draw parameters.
 - Fixed a potential `GL_INVALID_ENUM` error generated at initialization.
 - Glium will now panic if an OpenGL error is triggered during initialization.
 - Fixed gamma correction with OpenGL ES.
 - Fixed `get_internal_format_if_supported()` sometimes panicking.
 - Fixed a panic with a tessellation evaluation shader that outputs quads.

## Version 0.5.3

 - Added support for compute shaders with the `program::ComputeShader` struct.
 - Can now bind a uniform buffer as a shader storage buffer.
 - Added `Program::get_shader_storage_blocks()` to obtain the list of shader storage blocks.
 - The `Attribute` trait is now implemented on types from cgmath.
 - Now caching the actual format of a texture in case it is retrieved multiple times.

## Version 0.5.2

 - Added `vertex::TransformFeedbackSession` and `transform_feedback` to the draw parameters.
 - Added `RenderBufferAny`. Render buffers now deref to `RenderBufferAny`.
 - Deprecated `.into_vertex_buffer_any()` in favor of `.into()`.
 - Added `.get_total_bits()` to `texture::InternalFormat`.
 - Fixed values in uniform blocks being required to implement the `Default` trait.
 - Fixed buffer sometimes not always correctly unbound.

## Version 0.5.1 (2015-05-30)

 - Added `index::DrawCommandsNoIndicesBuffer` for multidraw indirect drawing.
 - Fixed a potential panic when using a buffer.

## Version 0.5.0 (2015-05-27)

 - `IndexBuffer` now takes the type of indices as template parameter.
 - `IndexBuffer` now derefs to `BufferView`. Allows modifying the index buffer after it has been created.
 - Added `IndexBufferAny`.
 - Removed the `ToIndicesSource` and `IntoIndexBuffer` traits. The former is replaced with `Into<IndicesSource>`.
 - `PixelBuffer` now takes the type of pixels as template parameter.
 - Renamed `PixelBuffer::read` to `PixelBuffer::read_as_texture_2d`.
 - `PixelBuffer` now derefs to `BufferView`.
 - Added `BufferView::read_as_texture_1d` and `BufferView::read_as_texture_1d_if_supported`.
 - Reworked `TextureDataSink` traits to take a precise format.
 - Fixed a panic when destroying a buffer with persistent mapping.
 - Removed deprecated function `VertexBuffer::new_dynamic`.
 - It is now safe to call `mem::forget` on a `Mapping` object.

## Version 0.4.2 (2015-05-25)

 - Removed the `buffer::Builder` type as it was proven unsound.
 - Renamed `SubBuffer` to `BufferView`.
 - `Display`/`GlutinFacade` now derefs to `Context`.
 - Mapping a buffer now simply calls `glMapBuffer` again, instead of writing to a temporary buffer.
 - `glutin` is now an optional dependency (enabled by default).
 - Creating an index buffer now correctly uses a `GL_ELEMENT_ARRAY_BUFFER`.

## Version 0.4.1 (2015-05-22)

 - Added a `buffer::Builder` type to build multiple sub-buffers within the same buffer.
 - Added an `invalidate` method to buffers.
 - Updated glutin to 0.1.6.

## Version 0.4.0 (2015-05-17)

 - Removed support for using indices in RAM.
 - `IndexBuffer`, `VertexBuffer` and `UniformBuffer` now deref to `SubBuffer`/`SubBufferAny`.
 - Added a `get_internal_format_if_supported` method to textures.
 - Replaced the `IntoProgramCreationInput` trait with `Into<ProgramCreationInput>`.
 - The `VertexFormat` type is now a `Cow<'static, []>` instead of a `Vec`.
 - Updated cgmath dependency to version 0.2.

## Version 0.3.7 (2015-05-12)

 - Added `AnySamplesPassedQuery`, `PrimitivesGeneratedQuery` and `TransformFeedbackPrimitivesWrittenQuery`, and corresponding fields in `DrawParameters`.
 - The `samples_passed_query` draw parameter can now also take a `AnySamplesPassedQuery`.
 - Added a `condition` parameter to the draw parameters, allowing you to use conditional rendering.
 - Textures now have a `sampled()` method to make it easier to create a `Sampler`.
 - Added a `color_mask` member to the draw parameters.
 - Added `per_instance` and `per_instance_if_supported` to `VertexBufferAny`.

## Version 0.3.6 (2015-05-09)

 - Added `SamplesPassedQuery` and `TimeElapsedQuery` types. They can be passed to the draw parameters.
 - Buffers are no longer mapped with `GL_MAP_COHERENT_BIT`. Flushing is handled by glium.
 - Changed `Surface::clear` to take an additional optional `Rect` parameter that specifies the rectangle to clear.
 - Fixed the `program!` macro not usable with version numbers >= 256.
 - Added support for `GL_OES_depth_texture` and `GL_OES_packed_depth_stencil`.
 - Moved the content of the `render_buffer` module to `framebuffer`. `render_buffer` still exists but is deprecated.

## Version 0.3.5 (2015-05-02)

 - Glium now reexports glutin. You can access glutin with `glium::glutin`.
 - Fixed trying to retrieve uniform blocks on OpenGL ES 2.

## Version 0.3.4 (2015-04-28)

 - Added caching some uniform values in the `Program` struct to avoid calls to `glUniform`.
 - `VertexBuffer::dynamic` and `write` can now take a  `&[T]` as well as a `Vec<T>`.
 - `assert_no_error` now takes an optional user-defined string.
 - Fixed triggering an OpenGL error on initialization with non-compatibility contexts.
 - Added a better texture units assignment system when drawing, in order to avoid some redundant state changes.

## Version 0.3.3 (2015-04-24)

 - Removed the `gl_persistent_mapping` feature.
 - Creating a dynamic buffer now creates a persistent mapped buffer automatically if supported.
 - Creating a non-dynamic buffer now creates an immutable buffer. Modifying such a buffer is still possible but very expensive.
 - Deprecated the `new_dynamic` function in favor of `dynamic`.
 - Fixed zero-sized textures triggering OpenGL errors.
 - Added the possibility to add a debug string in the OpenGL commands queue.

## Version 0.3.2 (2015-04-23)

 - Added `vertex::EmptyVertexAttributes` and `vertex::EmptyInstanceAttributes` markers in order to draw without a vertex source.
 - Added more formats in `CompressedFormats` and `CompressedSrgbFormats`.
 - Fixed the `Mapping` objects not implementing `Sync`.
 - Fixed rendering to an sRGB texture not possible.
 - `GliumCreationError` now has a template parameter indicating the backend creation error type.
 - The `DisplayBuild` trait has a new associated type indicating the error type.
 - Fixed the debug output callback not being disabled when compiling with --release.

## Version 0.3.1 (2015-04-18)

 - Fixed lifetime-related issues when using the `uniform!` macro.

## Version 0.3.0 (2015-04-16)

 - Changed the `slice()` functions to take a `Range` instead of two parameters.
 - Added `SrgbTexture` and `CompressedSrgbTexture` types.
 - Added `SrgbFormat` and `CompressedSrgbFormat` enums.
 - `GL_FRAMEBUFFER_SRGB` is now enabled by default.

## Version 0.2.3 (2015-04-14)

 - Added a `program!` macro which builds a program. Glium chooses the right shaders depending on what the backend supports.
 - Added a `VertexBuffer::empty` method that creates a vertex buffer with uninitialized data.
 - Fixed drawing with offsets in vertex buffers different than 0 not permitted.
 - Changed transform feedback reflection API to be compatible with what OpenGL 4.4 or ARB_enhanced_layouts allow.
 - `VertexBuffer::new` can now take a slice as parameter.
 - Revert the fix for sRGB. `GL_FRAMEBUFFER_SRGB` is no longer enabled.

## Version 0.2.2 (2015-04-10)

 - Added support for backends that don't have vertex array objects (like OpenGL ES 2/WebGL).
 - Glium now has basic support for OpenGL ES 2/WebGL.
 - Added stencil operations in `DrawParameters`.
 - Fixed `GL_FRAMEBUFFER_SRGB` not being enabled, leading to different brightness depending on the target.
 - Fixed trailing commas not working in `implement_vertex!` and `uniform!`.

## Version 0.2.1 (2015-04-03)

 - Creating a texture with a specific format now properly checks for available extensions.

## Version 0.2.0 (2015-03-30)

 - Removed `PerInstanceAttributesBuffer` and `PerInstanceAttributesBufferAny`.
 - Added `per_instance` and `per_instance_if_supported` to `VertexBuffer`.
 - Removed the deprecated `index_buffer` module. Use the `index` module instead.
 - Fixed viewport dimensions on retina screens.
 - The `Backend` trait is now marked unsafe.

## Version 0.1.3 (2015-03-24) & 0.1.4 (2015-03-26)

 - Added the `Backend` and `Facade` traits. `Display` implements the `Facade` trait.
 - Changed all buffer/texture/etc. creation functions to take any type that implements `Facade` instead of a `Display`.
 - Added `GlutinWindowBackend` and `GlutinHeadlessBackend` that implement the `Backend` trait.
 - Changed the private `Context` struct to be public. This allows users to implement the `Facade` trait themselves.
 - Added an associated type to the `DisplayBuild` trait.
 - Fixed scissor boxing not being disabled before a blit.

## Version 0.1.2 (2015-03-20)

 - Fixed a memory leak with vertex array objects.
 - Fixed an issue where you couldn't reuse the same uniform values created with `uniform!` multiple times.

## Version 0.1.1 (2015-03-13)

 - Added `ToColorAttachment` trait implementation for `Texture2dMultisample`.
 - Added `Texture2dMultisample::as_surface` method.
 - Updated the crate for the new I/O.
 - Changed `VertexFormat` to take a `Cow<'static, str>` instead of a `String`.
 - Fixed a stack overflow in release mode.
 - Removed the `fence` argument from vertices, indices and uniforms sources. Fences are now directly gathered from buffers when drawing.
