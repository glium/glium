/*!

This crate allows you to easily load textures from images.

Usage:

```no_run
# extern crate glium_core;
# extern crate glium_image;
# fn main() {
# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
# let file = std::io::util::NullReader;
let texture: glium_core::Texture = glium_image::ImageLoad::from_image(&display, file);
# }
```

*/

#![feature(tuple_indexing)]
#![deny(warnings)]
#![deny(missing_doc)]

extern crate image;
extern crate glium_core;

/// Traits that allows you to load textures from images.
pub trait ImageLoad {
    /// Loads an image.
    fn from_image<R: Reader>(&glium_core::Display, R) -> Self;
}

impl ImageLoad for glium_core::Texture {
    fn from_image<R: Reader>(display: &glium_core::Display, reader: R) -> glium_core::Texture {
        use image::GenericImage;

        let image = match image::load(reader, image::PNG) {
            Ok(img) => img,
            Err(e) => fail!("{}", e)
        };

        let image = image.to_rgba();

        let dimensions = image.dimensions();

        let data = image.pixelbuf();
        let data: &[(u8, u8, u8, u8)] = unsafe { std::mem::transmute(data) };

        glium_core::Texture::new(display, data, dimensions.0 as uint, dimensions.1 as uint, 1, 1)
    }
}
