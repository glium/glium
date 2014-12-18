# glium

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
 - Be compatible with the lowest OpenGL version possible, but still use 4.5 functionalities if they are available.
 - Be compatible with both OpenGL and OpenGL ES.

## [Link to the documentation](http://tomaka.github.io/glium)

The documentation contains examples about how to use it.

## Features

Glium has four features:

 - `image` allows support for the `image` library, which allows easy creation of textures from different image formats.
 - `headless`, which enables headless building and testing.
 - `gl_extensions`
 - `gles_extensions`

If you disable both `gl_extensions` and `gles_extensions`, then only
the functionalities that are available in both OpenGL 3 and OpenGL ES 2 will be
available at compile-time.

Enabling either `gl_extensions` or `gles_extensions` will unlock these
functionalities, but they will trigger a `panic!` if used when they are
not available on the target hardware.
