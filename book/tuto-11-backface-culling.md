# Backface culling

Before going further, there's one more thing to know about 3D rendering.

After your vertex shader outputs the vertex coordinates on the screen,
each triangle can be in two possible situations:

 - Its three vertices are in clockwise order on the screen.
 - Its three vertices are in counter-clockwise order on the screen.

If you ever rotate a triangle in order to see its back, then it will be in the other category.

Therefore you can associate the face of the triangle you're seeing to a order on the screen.
For example if the triangle is clockwise, then you're seeing face A, and if the triangle is
counter-clockwise, then you're seeing face B.

When you draw a 3D models, there are faces that you don't need to draw: the faces that are inside
of the model. Models are usually seen from the outside, so it's not a problem if the inside
doesn't actually exist.

If you make sure that all triangles of your model are in counter-clockwise order when the outside
is facing the camera (which is the case for the teapot used in these tutorials), you can ask the
video card to automatically discard all triangles that are in clockwise order. This technique is
called *backface culling*. Your 3D modelling software usually ensures that this convention is
applied.

Most of the time this is purely an optimization. By discarding half of the triangles after the
vertex shader step, you reduce by half the number of fragment shader invocations. This can lead
to a pretty good speedup.

## Backface culling in glium

Using backface culling in glium just consists in modifying a variable in the `DrawParameters` that
you pass to the `draw` function.

Just replace:

```rust
let params = glium::DrawParameters {
    depth: glium::Depth {
        test: glium::DepthTest::IfLess,
        write: true,
        .. Default::default()
    },
    .. Default::default()
};
```

With:

```rust
let params = glium::DrawParameters {
    depth: glium::Depth {
        test: glium::DepthTest::IfLess,
        write: true,
        .. Default::default()
    },
    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
    .. Default::default()
};
```

However we are not going to enable this for the teapot because the model is not closed. You can
look through holes and not see anything inside. 3D models are usually entirely closed, but not
our teapot.
