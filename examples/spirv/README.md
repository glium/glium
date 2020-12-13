# Loading SPIR-V shaders with Glium

In this example, we start with a vertex and fragment shader in separate GLSL files, `shader.vert` and `shader.frag`.
Since we want to load SPIR-V shaders, we first compile the GLSL shaders to SPIR-V.
For that, we use `glslangValidator` which you can download [here](https://github.com/google/shaderc/blob/main/downloads.md).

We compile them to SPIR-V modules (`.spv` files) as follows:
```sh
glslangValidator -G shader.vert -o vert.spv
glslangValidator -G shader.frag -o frag.spv
```

Then we can load them in Glium using:
```rust
ProgramCreationInput::SpirV(SpirvProgram::from_vs_and_fs(
    SpirvEntryPoint { data: include_bytes!("vert.spv"), entry_point: "main" },
    SpirvEntryPoint { data: include_bytes!("frag.spv"), entry_point: "main" },
))
```

But SPIR-V also allows having multiple entry points in the same module.
For this example we link fragment and vertex shaders together:
```sh
spirv-link vert.spv frag.spv -o shader.spv
```
And then we load them from the same `.spv` file:
```rust
let spirv = SpirvEntryPoint { data: include_bytes!("shader.spv"), entry_point: "main" };
let program = glium::Program::new(
    &display,
    ProgramCreationInput::SpirV(SpirvProgram::from_vs_and_fs(spirv, spirv)),
).unwrap();
```

Note: It's not a problem that both entry points are named `main`, since they are distinguished by their shader type:
```sh
$ spirv-cross shader.spv --reflect | jq .entryPoints
[
  {
    "name": "main",
    "mode": "vert"
  },
  {
    "name": "main",
    "mode": "frag"
  }
]
```

But we could also rename the entry points:
```sh
glslangValidator -G shader.vert --source-entrypoint main -e main_vs -o vert.spv
glslangValidator -G shader.frag --source-entrypoint main -e main_fs -o frag.spv
spirv-link vert.spv frag.spv -o shader.spv
```

Then we would load it like this:
```rust
let data = include_bytes!("shader.spv");
ProgramCreationInput::SpirV(SpirvProgram::from_vs_and_fs(
    SpirvEntryPoint { data, entry_point: "main_vs" },
    SpirvEntryPoint { data, entry_point: "main_fs" },
))
```
