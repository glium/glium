# The event loop

Before we continue animating things we need to make a short detour to make animation possible.
To simplify things we were only ever rendering a single frame during startup.
To change this we need to alter the structure a little bit, but first we should put the code that creates the context into a separate function. We'll use the following function in the following tutorials:

```rust
fn create_window_and_display(title: &str) -> (window: winit::Window, display: glium::Display<WindowSurface>) {
    // First we start by opening a new Window
    let window_builder = WindowBuilder::new().with_title(title);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
    let config_template_builder = ConfigTemplateBuilder::new();
    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences right now
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();

    // Now we get the window size to use as the initial size of the Surface
    let (width, height): (u32, u32) = window.inner_size().into();
    let attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    // Finally we can create a Surface, use it to make a PossiblyCurrentContext and create the glium Display
    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
    let context_attributes = glutin::context::ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let current_context = Some(unsafe {
        gl_config.display().create_context(&gl_config, &context_attributes).expect("failed to create context")
    }).unwrap().make_current(&surface).unwrap();
    let display = glium::Display::from_context_surface(current_context, surface).unwrap();

    (window, display)
}
```