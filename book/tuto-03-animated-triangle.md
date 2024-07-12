# Animating our triangle

Now that we have a triangle, we are going to animate it. Remember that OpenGL is like a drawing software. If we want to make a change on the screen, we have to draw over the existing content to replace what is already there.

So far we have only ever rendered a single frame and then waited for the program to exit. For an animation to show we need to change the way we draw our triangle. Instead of drawing a frame and then waiting in our event_loop for the window to close, we first draw our triangle when requested by the operating system:

```rust
let _ = event_loop.run(move |event, window_target| {
    match event {
        glium::winit::event::Event::WindowEvent { event, .. } => match event {
            glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
            glium::winit::event::WindowEvent::RedrawRequested => {
                // Move the draw code here!
            },
            _ => (),
        },
        _ => (),
    };
});
```

What exactly triggers this event is platform specific, but in order to draw our triangle over and over again we can request a redraw ourselves once we've finished rendering, to do that we'll respond to yet another event:

```rust
let _ = event_loop.run(move |event, window_target| {
    match event {
        glium::winit::event::Event::WindowEvent { event, .. } => match event {
            glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
            glium::winit::event::WindowEvent::RedrawRequested => {
                // Move the draw code here!
            },
            _ => (),
        },
        glium::winit::event::Event::AboutToWait => {
            window.request_redraw();
        },
        _ => (),
    };
});
```

There are other ways to render a scene but this is the preferred way for glutin/winit making it a good default choice.

While we are working on our event_loop there is one more event that we should handle, and that is a resize. Since glium only really has an OpenGL context we need to tell glium when the size of the underlying window has changed, otherwise you might see a streched image or borders. This is quite easy to accomplish:

```rust
let mut t: f32 = 0.0;
let _ = event_loop.run(move |event, window_target| {
    match event {
        glium::winit::event::Event::WindowEvent { event, .. } => match event {
            glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
            glium::winit::event::WindowEvent::Resized(window_size) => {
                display.resize(window_size.into());
            },
            glium::winit::event::WindowEvent::RedrawRequested => {
                // Move the draw code here!
            },
            _ => (),
        },
        glium::winit::event::Event::AboutToWait => {
            window.request_redraw();
        },
        _ => (),
    };
});
```

Now we can start animating our triangle!

# The naive approach

Our first approach will be to create a variable named `t` which represents the step in the animation. We update the value of `t` at each loop, and add it to the coordinates of our triangle at each frame:

```rust
let mut t: f32 = 0.0;
let _ = event_loop.run(move |event, window_target| {
    match event {
        glium::winit::event::Event::WindowEvent { event, .. } => match event {
            glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
            glium::winit::event::WindowEvent::Resized(window_size) => {
                display.resize(window_size.into());
            },
            glium::winit::event::WindowEvent::RedrawRequested => {
                // We update `t`
                t += 0.02;
                // We use the sine of t as an offset, this way we get a nice smooth animation
                let x_off = t.sin() * 0.5;

                let shape = vec![
                    Vertex { position: [-0.5 + x_off, -0.5] },
                    Vertex { position: [ 0.0 + x_off,  0.5] },
                    Vertex { position: [ 0.5 + x_off, -0.25] }
                ];
                let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 1.0, 1.0);
                target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                        &Default::default()).unwrap();
                target.finish().unwrap();
            },
            _ => (),
        },
        glium::winit::event::Event::AboutToWait => {
            window.request_redraw();
        },
        _ => (),
    };
});
```

If you run this code, you should see your triangle going from the left to the right and back again smoothly!

This method is approximately what game programmers were doing in the 1990s. This works perfectly fine when you have small shapes (like a single triangle), but it is highly inefficient when you manipulate models with thousands of polygons. There are two reasons for this:

 - The CPU would spend a lot of time calculating the coordinates every time you draw (with one operation for each vertex for each model, at the end you reach hundreds of thousands of operations).

 - It takes some time to upload our shape from RAM to video memory. This time is wasted as the GPU has to wait until the transfer is finished before it can start drawing.

# Uniforms

Do you remember vertex shaders? Our vertex shader takes as input the attributes of each vertex, and outputs its position on the window. Instead of doing the addition in our program and upload the result, we are going to ask the GPU to do this operation.

Let's remove the two `let`'s that redefine our shape and vertex_buffer from our draw handler:

```rust
let mut t: f32 = 0.0;
let _ = event_loop.run(move |event, window_target| {
    match event {
        glium::winit::event::Event::WindowEvent { event, .. } => match event {
            glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
            glium::winit::event::WindowEvent::Resized(window_size) => {
                display.resize(window_size.into());
            },
	    glium::winit::event::WindowEvent::RedrawRequested => {
	        // We update `t`
	        t += 0.02;
                // We use the sine of t as an offset, this way we get a nice smooth animation
	        let x_off = t.sin() * 0.5;

	        let mut target = display.draw();
	        target.clear_color(0.0, 0.0, 1.0, 1.0);
	        target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
			    &Default::default()).unwrap();
	        target.finish().unwrap();
	    },
            _ => (),
        },
        glium::winit::event::Event::AboutToWait => {
            window.request_redraw();
        },
        _ => (),
    };
});
```

And now we are going to change our vertex shader a little bit:

```rust
let vertex_shader_src = r#"
    #version 140

    in vec2 position;

    uniform float x;

    void main() {
        vec2 pos = position;
        pos.x += x;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"#;
```

You may notice that this is exactly the operation that we've been doing above, except that this time it is done on the GPU side. We have added a variable `t` in our shader, which is declared as a **uniform**. A uniform is a global variable whose value is set when we draw by passing its value to the `draw` function. The easiest way to do so is to use the `uniform!` macro:

```rust
target.draw(&vertex_buffer, &indices, &program, &uniform! { x: x_off },
            &Default::default()).unwrap();
```

Using uniform variables solves our two problems above. The CPU doesn't have to do any calculation, and all that it uploaded is the value of `x_off` (a single float) instead of the whole shape.
