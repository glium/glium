# Creating a project

To start this tutorial, we will create a new project from scratch. Even though it is highly recommended to be familiar with Rust and Cargo before starting, some little reminders are always good. Let's start by running:

```sh
cargo new --bin my_project
cd my_project
```

The directory you have just created should contain a `Cargo.toml` file which contains our project's metadata, plus a `src/main.rs` file which contains the Rust source code. If you have `src/lib.rs` file instead, that means that you forgot the `--bin` flag ; just rename `lib.rs` to `main.rs` then.

In order to use glium, you need to add it as a dependency to your `Cargo.toml` file.

```toml
[dependencies]
glium = "0.33"
```

By default glium pulls in everything necessary to get you started. You should now have a `src/main.rs` file that looks similar to this:

```rust
fn main() {
    println!("Hello, World!");
}
```

It is now time to start work on the main function!

# Creating a window

The first step when creating a graphical application is to create a window. If you have ever worked with OpenGL before, you know how hard it is to do this correctly. Both window creation and context creation are platform-specific, and they are sometimes complex and tedious.

Initializing a simple OpenGL window with the default winit/glutin backend can be done via the following 3 steps:

1. Creating an `EventLoop` for handling window and device events.
2. Making a new `SimpleWindowBuilder` and setting the desired parameters.
3. Calling the `build` method of the `SimpleWindowBuilder` with a reference to the event_loop to get the `Window` and `Display`.

This will open a new window, register it with the given event_loop and create a (glutin) OpenGL context and glium Display while finally returning both the window and display to you.

```rust
fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build().expect("event loop building");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
}
```

If you try to run this example with `cargo run` you'll encounter a problem: as soon as the window has been created, our main function exits and the window is closed. To prevent this, we need to wait until we receive a `CloseRequested` event. We do this by calling `event_loop.run`:

```rust
let _ = event_loop.run(move |event, window_target| {
    match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
	    winit::event::WindowEvent::CloseRequested => window_target.exit(),
	    _ => (),
        },
        _ => (),
    };
});
```

If you run the program now you should see an nice little window. The contents of the window, however, are not not very appealing. Depending on your system, it can appear black, show a random image, or just noise. We are expected to draw on the window, so the system doesn't initialize its color to a specific value.

# Drawing on the window

Glium and the OpenGL API work similarly to drawing software like Paint or GIMP. We begin with an empty image, then draw an object on it, then another object, then another object, etc. until we are satisfied with the result. But contrary to these programs, you don't want your users to see the intermediate steps. Only the final result should be displayed.

To accomplish this, OpenGL uses what is called *double buffering*. Instead of drawing directly on the window, we are drawing to an image stored in memory. Once we are finished drawing, this image is copied into the window.
This is represented in glium by the `Frame` object. When you want to start drawing something on a window, you must first call `display.draw()` to produce a new `Frame`:

```rust
let mut target = display.draw();
```

We can then use this `target` as a drawing surface. One of the operations that OpenGL and glium provide is filling the surface with a given color. This is what we are going to do.

```rust
target.clear_color(0.0, 0.0, 1.0, 1.0);
```

Note that to use this function, we will need to import the `Surface` trait first:

```rust
use glium::Surface;
```

The four values that we pass to `clear_color` represent the four components of our color: red, green, blue and alpha. Only values between `0.0` and `1.0` are valid. Here we are drawing an opaque blue color.

As explained above, the user doesn't immediately see the blue color on the screen. At this point if we were in a game, we would most likely draw our characters, their weapons, the ground, the sky, etc. But in this tutorial we will just stop here:

```rust
target.finish().unwrap();
```

This call to `finish()` means that we have finished drawing. It destroys the `Frame` object and copies our background image to the window. Our window is now filled with blue.

Here is our full program:

```rust
use glium::Surface;

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build().expect("event loop building");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let mut frame = display.draw();
    frame.clear_color(0.0, 0.0, 1.0, 1.0);
    frame.finish().unwrap();

    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
	        winit::event::WindowEvent::CloseRequested => window_target.exit(),
	        _ => (),
            },
            _ => (),
        };
    });
}
```
**[You can find the entire source code for this example here](https://github.com/glium/glium/blob/master/examples/tutorial-01.rs)**
