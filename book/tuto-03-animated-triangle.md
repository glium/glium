# Animating our triangle

Now that we have a triangle, we are going to try animating it. Remember that OpenGL is like a drawing software. If we want to make a change on the screen, we have to draw over the existing content to replace what is already there. Fortunately we already have a `loop` that continuously draws on the window, so our changes will almost instantly be reflected on the window.

# The naive approach

Our first approach will be to create a variable named `t` which represents the step in the animation. We update the value of `t` at each loop, and add it to the coordinates of our triangle at each frame:

```rust
let mut t: f32 = -0.5;
let mut closed = false;
while !closed {
    // we update `t`
    t += 0.0002;
    if t > 0.5 {
        t = -0.5;
    }

    // we create the shape an add `t` to each x coordinate
    let vertex1 = Vertex { position: [-0.5 + t, -0.5] };
    let vertex2 = Vertex { position: [ 0.0 + t,  0.5] };
    let vertex3 = Vertex { position: [ 0.5 + t, -0.25] };
    let shape = vec![vertex1, vertex2, vertex3];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    // drawing
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                &Default::default()).unwrap();
    target.finish().unwrap();

    events_loop.poll_events(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => ()
            },
            _ => (),
        }
    });
}
```

If you run this code, you should see your triangle going from the left to the right of the screen, then jumping back to the left!

This method is approximately what game programmers were doing in the 1990s. This works perfectly fine when you have small shapes (like a single triangle), but it is highly inefficient when you manipulate models with thousands of polygons. There are two reasons for this:

 - The CPU would spend a lot of time calculating the coordinates every time you draw (with one operation for each vertex for each model, at the end you reach hundreds of thousands of operations).

 - It takes some time to upload our shape from the RAM to the video memory. This time is totally wasted as the GPU has to wait until the transfer is finished to start its work.

# Uniforms

Do you remember vertex shaders? Our vertex shader takes as input the attributes of each vertex, and outputs its position on the window. Instead of doing the addition in our program and upload the result, we are going to ask the GPU to do this operation.

Let's reset our program to what it was at the end of the first tutorial, but keep `t`:

```rust
let vertex1 = Vertex { position: [-0.5, -0.5] };
let vertex2 = Vertex { position: [ 0.0,  0.5] };
let vertex3 = Vertex { position: [ 0.5, -0.25] };
let shape = vec![vertex1, vertex2, vertex3];

let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

let mut t: f32 = -0.5;
let mut closed = false;
while !closed {
    // we update `t`
    t += 0.0002;
    if t > 0.5 {
        t = -0.5;
    }

    // drawing
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                &Default::default()).unwrap();
    target.finish().unwrap();

    events_loop.poll_events(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => ()
            },
            _ => (),
        }
    });
}
```

And instead we are going to do a small change in our vertex shader:

```rust
let vertex_shader_src = r#"
    #version 140

    in vec2 position;

    uniform float t;

    void main() {
        vec2 pos = position;
        pos.x += t;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"#;
```

You may notice that this is exactly the operation that we've been doing above, except that this time it is done on the GPU side. We have added a variable `t` in our shader, which is declared as a **uniform**. A uniform is a global variable whose value is set when we draw by passing its value to the `draw` function. The easiest way to do so is to use the `uniform!` macro:

```rust
target.draw(&vertex_buffer, &indices, &program, &uniform! { t: t },
            &Default::default()).unwrap();
```

Using uniform variables solves our two problems above. The CPU doesn't have to do any calculation, and all that it uploaded is the value of `t` (a single float) instead of the whole shape.
