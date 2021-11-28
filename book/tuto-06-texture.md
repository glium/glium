
# Uploading a texture

A texture is an image or a collection of images loaded in the video memory.

In order to load a texture, we must first decode the image format that stores our image (for example, PNG). To do so, we are going to use the `image` library. Let's add it to the Cargo.toml file:

```toml
[dependencies]
image = "*"
```

And to the crate root:

```rust
extern crate image;
```

In order to load the image, we just need to use `image::load`:

```rust
use std::io::Cursor;
let image = image::load(Cursor::new(&include_bytes!("/path/to/image.png")),
                        image::ImageFormat::Png).unwrap().to_rgba8();
let image_dimensions = image.dimensions();
let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
```

And in order to upload the image as a texture, it's as simple as:

```rust
let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
```

# Using the texture

There is no automatic way to display a texture over a shape with OpenGL. Just like any other rendering techniques, it must be done manually. This means that we must manually load color values from our texture and return them with our fragment shader.

To do so, we first have to modify a bit our shape in order to indicate to which location of the texture each vertex is attached to:

```rust
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],       // <- this is new
}

implement_vertex!(Vertex, position, tex_coords);        // don't forget to add `tex_coords` here

let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] };
let vertex2 = Vertex { position: [ 0.0,  0.5], tex_coords: [0.0, 1.0] };
let vertex3 = Vertex { position: [ 0.5, -0.25], tex_coords: [1.0, 0.0] };
let shape = vec![vertex1, vertex2, vertex3];
```

Texture coordinates range from `0.0` to `1.0`. The coordinates `(0.0, 0.0)` correspond to the bottom-left hand corner of the texture, and `(1.0, 1.0)` to the top-right hand corner.

This new `tex_coords` attribute will be passed to the vertex shader, just like `position`. We don't have anything to do with it, and we are just going to pass it through to the fragment shader:

```glsl
#version 140

in vec2 position;
in vec2 tex_coords;
out vec2 v_tex_coords;

uniform mat4 matrix;

void main() {
    v_tex_coords = tex_coords;
    gl_Position = matrix * vec4(position, 0.0, 1.0);
}
```

Similarly to the `my_attr` variable, the value of `v_tex_coords` will be interpolated so that each pixel gets a value that corresponds to its position. This value corresponds here to the coordinates in the texture that this pixel is attached to.

All that's left to do in our fragment shader is to get the value of the color at these coordinates in the texture with the `texture()` function that is provided by OpenGL.

```glsl
#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;

void main() {
    color = texture(tex, v_tex_coords);
}
```

As you can see, a texture is a `uniform` of type `sampler2D`. There are many types of textures and texture uniforms, and `sampler2D` corresponds to a simple two-dimensional texture.

Since the texture is a uniform, we have to pass a reference to it when drawing in the Rust code:

```rust
let uniforms = uniform! {
    matrix: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [ t , 0.0, 0.0, 1.0f32],
    ],
    tex: &texture,
};
```

And here is the result:

![The result](resources/tuto-06-texture.png)

**[You can find the entire source code here](https://github.com/glium/glium/blob/master/examples/tutorial-06.rs).**
