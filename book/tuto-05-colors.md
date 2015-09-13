# Attributes

In our programming pipeline, the color of each pixel inside the triangle corresponds to the output of our fragment shader. Since our fragment shader returns `(1.0, 0.0, 0.0, 1.0)`, each pixel is an opaque red (the four values correspond to: red, green, blue, alpha/opacity).

In order to output the correct color, we need to have some information about the pixel we are trying to draw. Fortunately, it is possible to pass informations between the vertex and the fragment shader.

To do so, we simply add an `out` variable in the vertex shader...

```glsl
#version 140

in vec2 position;
out vec2 my_attr;      // our new attribute

uniform mat4 matrix;

void main() {
    my_attr = position;     // we need to set the value of each `out` variable.
    gl_Position = matrix * vec4(position, 0.0, 1.0);
}
```

...and an `in` variable with the same name and type in the fragment shader.

```glsl
#version 140

in vec2 my_attr;
out vec4 color;

void main() {
    color = vec4(my_attr, 0.0, 1.0);   // we build a vec4 from a vec2 and two floats
}
```

Let's see what happens. Our vertex shader is invoked three times, once per vertex. Each vertex returns a different value for `my_attr`. OpenGL then determines which pixels are inside the triangle during the rasterization phase, and calls the fragment shader once for each of these pixels. The value of `my_attr` that is passed for each pixel is **the interpolation of this value depending on the position of the pixel**.

For example, pixels that are right next to a vertex will get a value of `my_attr` that is equal or very near the value of `my_attr` that the vertex shader returned for this vertex. The pixel that is on the middle of the edge between two vertices will get the average of the two values of `my_attr` returned by the vertex shader for these two vertices. Pixels that are the middle of the triangle will get the average of the values of the three vertices.

*Note: this is because variables have by default the `smooth` attribute, which is what you want most of the time. It is also possible to specify the `flat` attribute.*

In the example above, the value of `my_attr` returned by the vertex shader corresponds to the position of the vertex. Therefore the value of `my_attr` that the fragment shader will get corresponds to the position of the pixel being processed. For the demonstration, we turn this position into the red and green components of our color.

And the result should look like this:

![The result](tuto-05-linear.png)

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
let image = image::load(Cursor::new(&include_bytes!("/path/to/image.png")[..]),
                        image::PNG).unwrap();
```

And in order to upload the image as a texture, it's as simple as:

```rust
let texture = glium::texture::Texture2d::new(&display, image).unwrap();
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

![The result](tuto-05-texture.png)

**[You can find the entire source code here](https://github.com/tomaka/glium/blob/master/examples/tutorial-05b.rs).**
