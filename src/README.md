# Implementor's notes

- Everthing related to OpenGL is executed on a dedicated thread that is spawned at initialization.

- The API sends commands to this OpenGL thread. To do so, call `display.context.context.exec`.
  Obtaining feedback (for example obtaining the result of a `glGen*` function) is done with
  regular Rust channels until a better solution is found.

- Commands sent by the API are executed in the same order as they are sent. This means that
  there is no problem in sending a command that uses a texture, then immediatly after a command
  that destroys the given texture.

- It is the responsibility of each function to ensure that the OpenGL functions being called and
  the constants being used are available in the current context, either with `if`s or with
  assertions.
