# Attributes

In our programming pipeline, the color of each pixel inside the triangle corresponds to the output of our fragment shader. Since our fragment shader returns `(1.0, 0.0, 0.0, 1.0)`, each pixel is an opaque red (the four values correspond to: red, green, blue, alpha/opacity).

In order to output the correct color, we need to have some information about the pixel we are trying to draw. Fortunately, it is possible to pass information between the vertex and the fragment shader.

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

**[You can find the entire source code here](https://github.com/tomaka/glium/blob/master/examples/tutorial-05.rs).**
