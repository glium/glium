# glium

[![Build Status](https://travis-ci.org/tomaka/glium.svg?branch=master)](https://travis-ci.org/tomaka/glium)

High-level OpenGL wrapper for the Rust language.

```toml
[dependencies.glium]
git = "https://github.com/tomaka/glium"
[dependencies.glium_macros]
git = "https://github.com/tomaka/glium"
```

Its objectives:
 - Be 100 % safe to use.
 - Avoid all GL errors. If a GL error is triggered, then it's a bug.
 - Provide all the features that core OpenGL provides.
 - Be compatible with the lowest OpenGL version possible, but still use 4.5 features if they are available.
 - Be compatible with both OpenGL and OpenGL ES.

## [Link to the documentation](http://tomaka.github.io/glium/glium/index.html)

The documentation contains examples showing how to use Glium.

## Why should I use Glium instead of raw OpenGL calls?

Easy to use:

 - Functions are higher level in Glium than in OpenGL. Glium's API tries to be as Rusty as
   possible, and shouldn't be much different than using any other Rust library.

 - You can directly pass vectors, matrices and images to Glium instead of manipulating low-level
   data.

 - Glium is designed to be very easy to setup.

 - Glium should allow you to do everything that OpenGL allows you to do, just through high-level
   functions. If something is missing, please open an issue.

 - Glium provides easier ways to do common tasks. For example the `VertexBuffer` struct
   contains information about the vertex bindings, because you usually don't use several different
   bindings with the same vertex buffer. Creating a program doesn't require creating shaders,
   attaching them, and linking - instead it is one single function call. Drawing on a texture
   is as easy as drawing on the backbuffer.

 - Glium is stateless. There are no `set_something()` functions in the entire library, and
   everything is done by parameter passing. The same set of function calls will always produce
   the same results.

 - Glium handles framebuffer objects, samplers, and vertex array objects for you. You no longer
   need to create them explicitely as they are automatically created when needed and destroyed
   when their corresponding object is destroyed.

Safety:

 - Glium detects what would normally be errors or undefined behaviors in OpenGL, and panics,
   without calling `glGetError` which would be too slow. Examples include requesting a depth test
   when you don't have a depth buffer available, not binding any value to an attribute or uniform,
   or binding multiple textures with different dimensions to the same framebuffer.

 - If the OpenGL context triggers an error, then you have found a bug in Glium. Please open
   an issue. Just like Rust does everything it can to avoid crashes, Glium does everything
   it can to avoid OpenGL errors.

 - Glium uses RAII. Creating a `Texture2d` struct creates a texture, and destroying the struct
   destroys the texture. It also uses Rust's lifetime system to ensure that objects are still
   alive when you use them. Glium provides the same guarantees with OpenGL objects that you have
   with regular objects in Rust.

 - High-level functions are much easier to use and thus less error-prone. For example there is
   no risk of making a mistake while specifying the names and offsets of your vertex attributes,
   since Glium automatically generates this data for you.

 - You can access the same Glium context from multiple threads simultaneously without
   having to worry about thread-safety.

Compatibility:

 - In its default mode, Glium should be compatible with both OpenGL and OpenGL ES. If something
   doesn't work on OpenGL ES, please open an issue.

 - During initialization, Glium detects whether the context provides all the required
   functionality, and returns an `Err` if the device is too old. Glium tries to be as tolerant
   as possible, and should work with the majority of the OpenGL2-era devices.

 - Glium will attempt to use the latest, optimized versions of OpenGL functions. This includes
   buffer and texture immutable storage and direct state access. It will automatically fall back
   to older functions if they are not available.

 - Glium comes with a set of tests that you can run with `cargo test`. If your project/game
   doesn't work on specific hardware, you can try running Glium's tests on it to see what is wrong.

Performances:

 - State changes are optimized. The OpenGL state is only modified if the state actually differs.
   For example if you call `draw` with the `IfLess` depth test twice in a row, then
   `glDepthFunc(GL_LESS)` and `glEnable(GL_DEPTH_TEST)` will only be called the first time. If
   you then call `draw` with `IfGreater`, then only `glDepthFunc(GL_GREATER)` will be called.

 - Just like Rust is theoretically slower than C because of additional safety checks, Glium is
   theoretically slower than well-prepared and optimized raw OpenGL calls. However in practice
   the difference is very low, if not negligible.

## Features

Glium has four Cargo features:

 - `image` allows support for the `image` library, which allows easy creation of textures from different image formats.
 - `cgmath` and `nalgebra` add support for these libraries' matrices and vectors.
 - `headless`, which enables headless building and testing.

In addition to this, it has the following OpenGL-related features:

 - `gl_read_buffer` (read the content of a buffer)
 - `gl_uniform_blocks` (bind buffers to uniform blocks)
 - `gl_sync` (synchronization objects)
 - `gl_persistent_mapping` (buffers permanently mapped in memory)
 - `gl_program_binary` (cache a compiled program in order to reload it faster next time)
 - `gl_tessellation` (ask the GPU to split primitives into multiple sub-primitives when rendering)
 - `gl_instancing` (draw multiple times the same model in only one command)

Enabling each of these features adds more restrictions towards the backend and increases the
likehood that `build_glium` will return an `Err`. However, it also gives you access to more
functions with different signatures.
