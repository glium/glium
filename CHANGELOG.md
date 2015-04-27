# Change Log

## Unreleased

 - Added caching some uniform values in the `Program` struct to avoid calls to `glUniform`.
 - `VertexBuffer::dynamic` and `write` can now take a  `&[T]` as well as a `Vec<T>`.
 - `assert_no_error` now takes an optional user-defined string.

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
