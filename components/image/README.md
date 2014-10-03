# glium_image

This crate allows you to easily load textures from images.

```toml
[dependencies.glium_image]
git = "https://github.com/glium/glium_image"
```

Usage:

```rust
let texture: glium_core::Texture = glium_core_image::ImageLoad::from_image(&display, file);
```
