/*!

Usage:

```no_run
# extern crate glium_core;
# extern crate glium_sprite2d;
# fn main() {
# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
# }
```

*/

#![feature(phase)]
#![feature(tuple_indexing)]
#![deny(missing_doc)]
#![deny(warnings)]

extern crate cgmath;
extern crate glium_core;
extern crate glium_sprite2d;
extern crate spine;

use cgmath::FixedArray;

/// Object that will allow you to draw spine documents with `glium_core`.
pub struct Spine {
    textures: Vec<(String, glium_core::Texture)>,
    document: spine::SpineDocument,
}

impl Spine {
    /// Builds a new `Spine`.
    ///
    /// TODO: handle errors
    pub fn new<R: Reader>(spine_doc: R, resources_loader: |&str| -> glium_core::Texture)
        -> Spine
    {
        let document = spine::SpineDocument::new(spine_doc).unwrap();

        let textures = document.get_possible_sprites().into_iter()
            .map(|tex| (tex.to_string(), resources_loader(tex))).collect();

        Spine {
            textures: textures,
            document: document,
        }
    }
}

impl Deref<spine::SpineDocument> for Spine {
    fn deref(&self) -> &spine::SpineDocument {
        &self.document
    }
}

/// 
pub struct SpineDraw<'a, 'b, 'c, 'd, 'e, 'f>(pub &'a Spine, pub &'b glium_sprite2d::Sprite2DSystem<'c>,
    pub &'d str, pub Option<&'e str>, pub f32, pub &'f cgmath::Matrix4<f32>);

impl<'a, 'b, 'c, 'd, 'e, 'f> glium_core::DrawCommand for SpineDraw<'a, 'b, 'c, 'd, 'e, 'f> {
    fn draw(self, target: &mut glium_core::Target) {
        let results = match self.0.document.calculate(self.2, self.3, self.4) {
            Ok(r) => r,
            Err(_) => return
        };

        for (sprite, matrix, _color) in results.sprites.into_iter() {
            let texture = match self.0.textures.iter()
                .find(|&&(ref name, _)| name.as_slice() == sprite)
            {
                Some(tex) => &tex.1,
                None => continue
            };

            target.draw(glium_sprite2d::SpriteDisplay {
                sprite: self.1,
                texture: texture,
                matrix: &(*self.5 * matrix).into_fixed(),
            });
        }
    }
}
