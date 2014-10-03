# glium-sprite2d

```toml
[dependencies.glium_sprite2d]
git = "https://github.com/glium/glium-sprite2d"
```

## Usage

```rust
// during initialization
let sprite2d_sys = glium_sprite2d::Sprite2DSystem::new(&display);

// when drawing
target.draw(&glium_sprite2d::SpriteDisplay {
    sprite: &sprite2d_sys,
    texture: &texture,
    matrix: &[
        [ 1.0, 0.0, 0.0, 0.0 ],
        [ 0.0, 1.0, 0.0, 0.0 ],
        [ 0.0, 0.0, 1.0, 0.0 ],
        [ 0.0, 0.0, 0.0, 1.0 ]
    ]
});
```
