#![cfg(feature = "simple_window_builder")]
/*!

A utility to simplify context creation with glutin.

[`SimpleWindowBuilder`] is rather limited, as it targets trivial use cases and beginners.
Its [`build()`](SimpleWindowBuilder::build) method is a good starting point for a custom context
creation routine.

# Features

Only available if the 'simple_window_builder' feature is enabled.

*/
use super::*;
use crate::glutin::config::{Config, ConfigTemplateBuilder};
use crate::winit::window::Window;
use crate::winit::window::WindowAttributes;
use glutin_winit::DisplayBuilder;
use std::error::Error;
use std::num::NonZeroU32;

/// A `winit::event_loop::EventLoop`-like Window provider suitable for [`DisplayBuilder`].
///
/// This trait exists due to a technicality: older versions of `winit` required the use of
/// a `winit::event_loop::EventLoop` to construct a [`Window`], while starting with version 0.30.0
/// that method is [deprecated](winit::event_loop::EventLoop::create_window) in favor
/// of `winit::event_loop::ActiveEventLoop::create_window`. This forced the developers
/// of `glutin_winit` to introduce a trait just like this one.
///
/// Since `glutin_winit::event_loop::GlutinEventLoop` is not public, `glutin` has its own copy.
///
/// This trait should not be implemented for any other types.
pub trait GliumEventLoop {
    /// Calls `display_builder.build(self, template_builder, config_picker)`.
    fn build<Picker>(
        &self,
        display_builder: DisplayBuilder,
        template_builder: ConfigTemplateBuilder,
        config_picker: Picker,
    ) -> Result<(Option<Window>, Config), Box<dyn Error>>
    where
        Picker: FnOnce(Box<dyn Iterator<Item = Config> + '_>) -> Config,
        Self: Sized;
}

impl<T> GliumEventLoop for winit::event_loop::EventLoop<T> {
    fn build<Picker>(
        &self,
        display_builder: DisplayBuilder,
        template_builder: ConfigTemplateBuilder,
        config_picker: Picker,
    ) -> Result<(Option<Window>, Config), Box<dyn Error>>
    where
        Picker: FnOnce(Box<dyn Iterator<Item = Config> + '_>) -> Config,
    {
        display_builder.build(self, template_builder, config_picker)
    }
}

impl GliumEventLoop for winit::event_loop::ActiveEventLoop {
    fn build<Picker>(
        &self,
        display_builder: DisplayBuilder,
        template_builder: ConfigTemplateBuilder,
        config_picker: Picker,
    ) -> Result<(Option<Window>, Config), Box<dyn Error>>
    where
        Picker: FnOnce(Box<dyn Iterator<Item = Config> + '_>) -> Config,
    {
        display_builder.build(self, template_builder, config_picker)
    }
}

/// Builder to simplify glium/glutin context creation.
pub struct SimpleWindowBuilder {
    attributes: WindowAttributes,
    config_template_builder: ConfigTemplateBuilder,
    vsync: bool
}

impl SimpleWindowBuilder {
    /// Initializes a new builder with default values.
    pub fn new() -> Self {
        Self {
            attributes: Window::default_attributes()
                .with_title("Simple Glium Window")
                .with_inner_size(winit::dpi::PhysicalSize::new(800, 480)),
            config_template_builder: ConfigTemplateBuilder::new(),
            vsync: true

        }
    }

    /// Requests the window to be of a certain size.
    /// If this is not set, the builder defaults to 800x480.
    pub fn with_inner_size(mut self, width: u32, height: u32) -> Self {
        self.attributes = self
            .attributes
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height));
        self
    }

    /// Set the initial title for the window.
    pub fn with_title(mut self, title: &str) -> Self {
        self.attributes = self.attributes.with_title(title);
        self
    }

    /// Replace the used [`WindowAttributes`],
    /// do this before you set other parameters or you'll overwrite the parameters.
    pub fn set_window_builder(mut self, window_attributes: WindowAttributes) -> Self {
        self.attributes = window_attributes;
        self
    }

    /// Replace the used [`ConfigTemplateBuilder`],
    /// Can be used to configure among other things buffer sizes and number of samples for the window.
    pub fn with_config_template_builder(mut self, config_template_builder: ConfigTemplateBuilder) -> Self {
        self.config_template_builder = config_template_builder;
        self
    }

    /// Returns the inner [`WindowAttributes`].
    pub fn into_window_builder(self) -> WindowAttributes {
        self.attributes
    }

    /// Replace the used vsync configuration
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Create a new [`Window`] and [`Display`]
    /// with the specified parameters.
    pub fn build(
        self,
        event_loop: &impl GliumEventLoop,
    ) -> (
        Window,
        Display<glutin::surface::WindowSurface>,
    ) {
        use glutin::prelude::*;
        use raw_window_handle::HasWindowHandle;

        // First we start by opening a new Window
        let display_builder =
            DisplayBuilder::new().with_window_attributes(Some(self.attributes));
        let config_template_builder = ConfigTemplateBuilder::new();
        let (window, gl_config) = event_loop.build(display_builder, self.config_template_builder, |mut configs| {
                // Just use the first configuration since we don't have any special preferences here
                configs.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();

        // Now we get the window size to use as the initial size of the Surface
        let (width, height): (u32, u32) = window.inner_size().into();
        let attrs =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window.window_handle().expect("couldn't obtain raw window handle").into(),
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                );

        // Finally we can create a Surface, use it to make a PossiblyCurrentContext and create the glium Display
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            .build(Some(window.window_handle().expect("couldn't obtain raw window handle").into()));
        let current_context = Some(unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create context")
        })
        .unwrap()
        .make_current(&surface)
        .unwrap();

        let swap_interval = if self.vsync {
            glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap())
        } else {
            glutin::surface::SwapInterval::DontWait
        };
        surface.set_swap_interval(&current_context, swap_interval).unwrap();

        let display = Display::from_context_surface(current_context, surface).unwrap();

        (window, display)
    }
}
