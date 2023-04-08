# Creating a project

To start this tutorial, we will create a new project from scratch. Even though it's highly recommended to be familiar with Rust and Cargo before starting, some little reminders are always good. Let's start by running:

```sh
cargo new --bin my_project
cd my_project
```

The directory you have just created should contain a `Cargo.toml` file which contains our project's metadata, plus a `src/main.rs` file which contains the Rust source code. If you have `src/lib.rs` file instead, that means that you forgot the `--bin` flag ; just rename `lib.rs` to `main.rs` then.

In order to use glium, we need to add a couple of dependencies to our `Cargo.toml` file:

```toml
[dependencies]
winit = "0.28"
raw-window-handle = "0.5"
glutin = "0.30"
glutin-winit = "0.3"
glium = "0.33"
```

I'll explain what these libraries do as we use them to build your program. Your `src/main.rs` file should now look something like this:

```rust
fn main() {
    println!("Hello, world!");
}
```

It is now time to start filling in the `main` function!

# Creating a window

The first step when creating a graphical application is to create a window. If you have ever worked with OpenGL before, you know how hard it is to do this correctly. Both window creation and context creation are platform-specific, and they are sometimes weird and tedious.

We start off by using **winit** to open a window.

```rust
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let window_builder = WindowBuilder::new();

    let window = window_builder.build(&event_loop).unwrap();
}
```

If you try to run this example with `cargo run` you'll encounter  a problem: as soon as the window has been created, our main function exits and the window is closed. To prevent this, we need to loop until we receive a `CloseRequested` event. We do this by calling `event_loop.run`:

```rust
event_loop.run(move |ev, _, control_flow| {
    match ev {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => {
                *control_flow =  winit::event_loop::ControlFlow::Exit;
            },
            _ => (),
        },
        _ => (),
    }
});
```

If you run the program now you should see an nice little window. The content of the window, however, is not not very appealing. Depending on your system, it can appear black, show a random image, or just some snow. We are expected to draw on the window, so the system doesn't bother initializing its color to a specific value.

Since we want to draw using OpenGL we'll leave it like this for now.

# Making it OpenGL capable

To make use of OpenGL we first need a context, in these examples we'll be using **glutin** for that, we also need the **glutin-winit** and **raw-window-handle** crates to connect the context to our **winit** window.

First we import 2 new types above our main function:

```rust
use glutin_winit::DisplayBuilder;
use glutin::config::ConfigTemplateBuilder;
```

Now we can change the way we open our window to make it capable of supporting an OpenGL context. To accomplish that replace the `let window = ...` line with the following:

```rust
let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
let config_template_builder = ConfigTemplateBuilder::new();
let (window, gl_config) = display_builder
    .build(&event_loop, config_template_builder, |mut configs| {
        // Just use the first configuration since we don't have any special preferences right now
        configs.next().unwrap()
    })
    .unwrap();
let window = window.unwrap();
```

This might seem complicated but it allows for great flexibility in how the window is created which is necessary to support mobile platforms. If you run the program now you won't see a difference, but the window can now support an OpenGL context.

# Creating an OpenGL context

First off we need to use some more libraries, to do that add the following above the main function:

```rust
use glutin::prelude::*;
use glutin::display::GetGlDisplay;
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
```

To create a glium context we need a glutin `Surface`, to do that add the following after the `let window = ...` statement:

```rust
let (width, height): (u32, u32) = window.inner_size().into();
let attrs = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
    window.raw_window_handle(),
    NonZeroU32::new(width).unwrap(),
    NonZeroU32::new(height).unwrap(),
);
let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
```

Here we first get the size of our window and the create a `Surface` with those dimensions.

After that we can finally create a glutin context and glium `Display` by adding the following:

```rust
let context_attributes = glutin::context::ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
let current_context = Some(unsafe {
    gl_config.display().create_context(&gl_config, &context_attributes).expect("failed to create context")
}).unwrap().make_current(&surface).unwrap();
let display = glium::Display::from_context_surface(current_context, surface).unwrap();
```

Now we are ready to use glium.

# Drawing on the window

Glium and the OpenGL API work similarly to drawing software like Microsoft Paint or GIMP. We begin with an empty image, then draw an object on it, then another object, then another object, etc. until we are satisfied with the result. But contrary to drawing software, you don't want your users to see the intermediate steps. Only the final result should be shown.

To accomplish this, OpenGL uses what is called *double buffering*. Instead of drawing directly to the window, we are drawing to an image stored in memory. Once we are finished drawing, this image is copied into the window.
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

Like I explained above, the user doesn't immediately see the blue color on the screen. At this point if we were in a real application, we would most likely draw our characters, their weapons, the ground, the sky, etc. But in this tutorial we will just stop here:

```rust
target.finish().unwrap();
```

This call to `finish()` means that we have finished drawing. It destroys the `Frame` object and copies our background image to the window. Our window is now filled with blue.

Here is our full program:

```rust
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;
use glutin_winit::DisplayBuilder;
use glutin::config::ConfigTemplateBuilder;
use glutin::prelude::*;
use glutin::display::GetGlDisplay;
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use glium::Surface;

fn main() {
    let event_loop = EventLoopBuilder::new().build();

    let window_builder = WindowBuilder::new();
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
    let config_template_builder = ConfigTemplateBuilder::new();
    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences right now
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();

    let (width, height): (u32, u32) = window.inner_size().into();
    let attrs = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );
    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };

    let context_attributes = glutin::context::ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let current_context = Some(unsafe {
        gl_config.display().create_context(&gl_config, &context_attributes).expect("failed to create context")
    }).unwrap().make_current(&surface).unwrap();
    let display = glium::Display::from_context_surface(current_context, surface).unwrap();

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    target.finish().unwrap();

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow =  winit::event_loop::ControlFlow::Exit;
                },
                _ => (),
            },
            _ => (),
        }
    });
}
```
