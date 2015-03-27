# Change Log

## Unreleased

 - Removed the deprecated `index_buffer` module. Use the `index` module instead.

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
