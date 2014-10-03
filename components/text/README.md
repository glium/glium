# simple-gl-text

This crate allows you to easily write text with simple-gl.

Usage:

```rust
// The `TextSystem` contains the shaders and elements used for text display.
let system = glium_text::TextSystem::new(&display);

// Creating a `FontTexture`, which a regular `Texture` which contains the font.
// Note that loading the systems fonts is not covered by this library.
let font = glium_text::FontTexture::new(&display, std::io::File::open(&Path::new("my_font.ttf")), 24).unwrap();

// Creating a `TextDisplay` which contains the elements required to draw a specific sentence.
let text = glium_text::TextDisplay::new(&system, &font, "Hello world!");

// Finally, drawing the text is done with a `DrawCommand`.
// This draw command contains the matrix and color to use for the text.
display.draw().draw(glium_text::DrawCommand(&text,
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ], [1.0, 1.0, 0.0, 1.0]));
```
