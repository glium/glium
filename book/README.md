# Introduction

Hello and welcome to the glium tutorials! This series of tutorials will teach you how to work with OpenGL thanks to the glium library. Glium's API uses the exact same concepts as OpenGL and has been designed to remove the burden of using raw OpenGL function calls, which are often non-portable, tedious and error-prone. Even if for some reason you don't plan on using the glium library in the future, these tutorials can still be useful as they will teach you how OpenGL and graphics programming in general work.

If at any moment you encounter an error, please open an issue. Everything related to the window, fullscreen mode, or events is handled by [glutin](https://github.com/rust-windowing/glutin/issues), while everything related to rendering is handled by [glium](https://github.com/glium/glium/issues).

# Summary

* Tutorial

 * [Opening a window](tuto-01-getting-started.md)
 * [Drawing a triangle](tuto-02-triangle.md)
 * [Uniforms](tuto-03-animated-triangle.md)
 * [Matrices](tuto-04-matrices.md)
 * [Adding colors](tuto-05-colors.md)
 * [Adding a texture](tuto-06-texture.md)
 * [A more complex shape](tuto-07-shape.md)
 * [Gouraud shading](tuto-08-gouraud.md)
 * [Depth testing](tuto-09-depth.md)
 * [Adjusting the perspective](tuto-10-perspective.md)
 * [Backface culling](tuto-11-backface-culling.md)
 * [The camera and summary of the vertex processing stages](tuto-12-camera.md)
 * [Blinn-phong shading](tuto-13-phong.md)
 * [Normal mapping](tuto-14-wall.md)
 * Parallax mapping
 * Deferred shading
 * Shadow mapping
 * Antialiasing
 * Drawing lots of objects with instancing
 

* [Performances](perf-intro.md)

 * [Synchronization](perf-sync.md)
