---
layout: tutorials
title: Animating our triangle
---

Now that we have a triangle, we are going to try animating it. Remember that OpenGL is like a drawing software. If we want to make a change on the screen, we have to draw over the existing content to replace what is already there. Fortunately we already have a `loop` that continuously draws on the window, so our changes will almost instantly be reflected on the window.

# The naive approach

Our first approach will be to create a variable named `t` which represents the step in the animation. We update the value of `t` at each loop, and add it to the coordinates of our triangle at each frame:


    let mut t = -0.5;

    loop {
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
        let vertex_buffer = glium::VertexBuffer::new(&display, shape);

        // drawing
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                    &std::default::Default::default()).unwrap();
        target.finish();

        if display.is_closed() {
            break;
        }
    }

If you run this code, you should see your triangle going from the left to the right of the screen, then jumping back to the left!

This method is approximately what game programmers were doing in the 1990s. This works perfectly fine when you have small shapes (like a single triangle), but it is highly inefficient when you manipulate models with thousands of polygons. There are two reasons for this:

 1) The CPU would spend a lot of time calculating the coordinates every time you draw (with one operation for each vertex for each model, at the end you reach hundreds of thousands of operations).
 2) It takes some time to upload our shape from the RAM to the video memory. This time is totally wasted as the GPU has to wait until the transfer is finished to start its work.

# Uniforms

Do you remember vertex shaders? Our vertex shader takes as input the attributes of each vertex, and outputs its position on the window. Instead of doing the addition in our program and upload the result, we are going to ask the GPU to do this operation.

Let's reset our program to what it was at the end of the first tutorial, but keep `t`:

    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ 0.0,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, -0.25] };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&display, shape);

    let mut t = -0.5;

    loop {
        // we update `t`
        t += 0.0002;
        if t > 0.5 {
            t = -0.5;
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                    &std::default::Default::default()).unwrap();
        target.finish();

        if display.is_closed() {
            break;
        }
    }

And instead we are going to do a small change in our vertex shader:

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

You may notice that this is exactly the operation that we've been doing above, except that this time it is done on the GPU side. We have added a variable `t` in our shader, which is declared as a **uniform**. A uniform is a global variable whose value is set when we draw by passing its value to the `draw` function. The easiest way to do so is to use the `uniform!` macro:

    target.draw(&vertex_buffer, &indices, &program, &uniform! { t: t },
                &std::default::Default::default()).unwrap();

Using uniform variables solves our two problems above. The CPU doesn't have to do any calculation, and all that it uploaded is the value of `t` (a single float) instead of the whole shape.

# Matrices

We are moving our triangle from the left to the right of the screen with a simple addition. But what about other transformations like rotations, skews or rescalings?

All the geometrical operations that we need can be done with some maths:

 - Rescaling our triangle is done with `position *= factor;`
 - Rotating our triangle is done with `new_position = vec2(pos.x * cos(angle) - pos.y * sin(angle), pos.x * sin(angle) + pos.y * cos(angle));`
 - Skewing our triangle is done with `position.x += position.y * factor;`

But what if we want to do a rotation, then a translation, then a rescale? Or a skew and a rotation? Even though it's possible to do this with maths, things become very complex to handle.

Instead, programers use **matrices**. A matrix is a two-dimensional table of numbers which *can represent a geometrical transformation*. In computer graphics, we use 4x4 matrices.

Let's get back to our moving triangle. We are going to change the vertex shader to use a matrix. Instead of adding the value of `t` to the coordinates, we are going to apply the matrix to them by multiplying it. This applies the transformation described by our matrix to the vertex's coordinates.

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        uniform mat4 matrix;

        void main() {
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

Note that it is important to write `matrix * vertex` and not `vertex * matrix`. Matrix operations produce different result depending on the order.

We also need to pass the matrix when calling the `draw` function:

    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [ t , 0.0, 0.0, 1.0],
        ]
    };

    target.draw(&vertex_buffer, &indices, &program, &uniforms,
                &std::default::Default::default()).unwrap();

You should see exactly the same thing as previously, but what we now have is much more flexible. For example, if instead we want to rotate the triangle we can try this matrix instead:

    use std::num::Float;

    let uniforms = uniform! {
        matrix: [
            [ t.cos(), t.sin(), 0.0, 0.0],
            [-t.sin(), t.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    };

### Final source code

Here is the final code of our `src/main.rs` file:

    #[macro_use]
    extern crate glium;

    fn main() {
        use glium::{DisplayBuild, Surface};
        let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        let vertex1 = Vertex { position: [-0.5, -0.5] };
        let vertex2 = Vertex { position: [ 0.0,  0.5] };
        let vertex3 = Vertex { position: [ 0.5, -0.25] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = glium::VertexBuffer::new(&display, shape);
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let vertex_shader_src = r#"
            #version 140

            in vec2 position;

            uniform mat4 matrix;

            void main() {
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

        let mut t = -0.5;

        loop {
            // we update `t`
            t += 0.0002;
            if t > 0.5 {
                t = -0.5;
            }

            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);

            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [ t , 0.0, 0.0, 1.0],
                ]
            };

            target.draw(&vertex_buffer, &indices, &program, &uniforms,
                        &std::default::Default::default()).unwrap();
            target.finish();

            if display.is_closed() {
                break;
            }
        }
    }
