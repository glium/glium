# Performance

Here is the estimated cost of various operations:

 - **Creating a `Program`**: very high, as the driver has to compile the source code of the program.
   Do this only during initialization.

 - **Creating an empty buffer**: medium for vertex and index buffers, low for other buffers.

 - **Uploading data to a buffer**: low to medium. The transfer rate between RAM and video memory is
   around 15 GB per second today. This means that uploading 1 MB of data takes around 65µs. The
   OpenGL implementation can also sometimes give you a buffer whose data is located in RAM, in
   which case you pay the price when drawing rather than uploading.

 - **Copying between two buffers**: very low (similar to a memcpy on the CPU). This operation is done
   asynchronously by the GPU or the DMA. The transfer rate is around 50 GB per second.

 - **Creating a texture**: low. Reusing an existing texture is faster than creating a new one, but not
   by much.

 - **Uploading data to a texture**: low to high. If the data you give to OpenGL matches the texture's
   internal format then the cost is the same as uploading data to a buffer. However if the data
   has the wrong format then the GPU must perform conversions.

 - **Transferring between a pixel buffer and a texture**: very low to very high. This is similar to
   "Uploading data to a texture". If the data matches the texture's internal format, then it is
   simply a transfer between video memory. If the data doesn't match the format, then the OpenGL
   implementation will read from the texture/buffer to RAM, perform conversions, then upload the data to
   the texture/buffer.

 - **A draw call**: medium. A draw call has a fixed cost on the CPU, and both a fixed cost and a
   variable cost on the GPU. In order to reduce that fixed cost, you should group draw calls
   together if you can. For example drawing ten sprites is better done by writing twenty
   triangles in the same vertex buffer and submitting only one command, instead of submitting
   ten commands.

 - **Swapping buffers**: very low/variable. The process of swapping buffers at the end of a frame
   is very fast. However if you benchmark this function call you can probably see that it takes a
   a lot of time. The reason is that the OpenGL implementation usually doesn't send commands
   to the GPU immediately. Instead it adds commands to a local queue, then sends chunks of commands
   at once. When you swap buffers, the implementation flushes its local queue and sends all its
   commands to the GPU. In addition to this, also note that vsync can make swapping buffers block
   until the screen refreshes.

## Avoiding state changes

Doing multiple draw calls in a row with the same parameters (same vertex source, same program,
same draw parameters, same uniforms) is faster than switching parameters.

More precisely:

 - Changing uniforms of basic types (floats, integers, etc.) between two draw calls: low.

 - Changing texture uniforms between two draw calls: medium.

 - Changing the draw parameters between two draw calls: medium.

 - Changing the source of vertices between two draw calls: medium.

 - Changing the program between two draw calls: high.

 - Changing the render target between two draw calls: high.

Therefore if you have a lot of things to draw, you should group objects by program, draw parameters,
and vertex source.
