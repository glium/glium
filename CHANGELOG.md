# Change Log

## Unreleased

 - Added `ToColorAttachment` trait implementation for `Texture2dMultisample`.
 - Added `Texture2dMultisample::as_surface` method.
 - Changed `VertexFormat` to take a `Cow<'static, str>` instead of a `String`.
 - Fixed a stack overflow in release mode.
 - Removed the `fence` argument from vertices, indices and uniforms sources. Fences are now directly gathered from buffers when drawing.
