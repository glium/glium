# Synchronization

Almost all OpenGL implementations today are hardware-accelerated. This means that when you execute
OpenGL commands, it is in fact your video card that does the hard work instead of your CPU.

In order to improve performances, calling an OpenGL function does not wait for the operation
to be over. Instead it just sends a command and returns immediately. In a good application, the
CPU adds commands to a queue while the GPU reads them and processes them in parallel.

*Note: If the GPU processes commands faster than the CPU sends them, we say that the application
is CPU-bound. In the other case, the application is GPU-bound. AAA video games are almost always
GPU-bound.*

But there's a problem: in some situations there is no other choice but to wait for the commands to
finish. For example, if you read the contents of a texture which you have just
drawn to, there is technically no other choice but to wait for the rendering to finish before
reading. This is called a **synchronization**, because the CPU and the GPU must synchronize instead
of executing things in parallel.

It is your job, as an OpenGL programmer, to avoid all the costly operations that cause a
synchronization.

## Reading from a texture or from the framebuffer

In order to read a texture or the framebuffer without causing a synchronization, we have to use
a **pixel buffer**. Instead of directly reading the texture, we ask the GPU to copy its content
to a buffer, and we then read that buffer later on.

Just like any other operation, copying from the texture to the pixel buffer is a regular command
that the GPU will execute. If we wait long enough, the buffer will no longer be in use and we can
read from it without waiting.

## About write-only operations

One common operation that is often done in graphics programming is streaming data to the
GPU. In other words, you send data to a buffer or a texture just before using it. This is done
for example when rendering particles (which move a lot) or when decoding a video.

Since creating a buffer or a texture can be expensive, it is preferred to always use the same
buffer or texture and write data to it.

There are two possibilities here:

 - You rewrite the content of the entire buffer or texture.
 - You write only in some parts of the buffer or texture.

These two situations are very different. If you rewrite the entire buffer or texture, then the
OpenGL implementation is usually smart enough to actually allocate a new buffer or texture and put
your data in it instead of reusing the same memory. This is done in a totally transparent way.

Instead if you write only some parts of the buffer or texture, then the implementation can't do
that. This is where **invalidating** a buffer or a texture comes into play. By calling
`.invalidate()`, you tell the OpenGL implementation that you don't care about what was already
in the buffer. This allows it to use the same optimization as when you rewrite the entire
buffer or texture.
