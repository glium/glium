use context::CommandContext;
use version::Api;
use version::Version;

use DrawError;
use gl;

/// Represents the depth parameters of a draw command.
#[derive(Debug, Copy, Clone)]
pub struct Depth {
    /// The function that the GPU will use to determine whether to write over an existing pixel
    /// on the target. Don't forget to set `depth_write` appropriately if you use a depth test.
    ///
    /// See the `DepthTest` documentation for more details.
    ///
    /// The default is `Overwrite`.
    pub test: DepthTest,

    /// Sets whether the GPU will write the depth values on the depth buffer if they pass the
    /// depth test.
    ///
    /// The default is `false`. You most likely want `true` if you're doing depth testing.
    ///
    /// If you pass `true` but don't have a depth buffer available, drawing will produce
    /// a `NoDepthBuffer` error.
    pub write: bool,

    /// The range of possible Z values in surface coordinates.
    ///
    /// Just like OpenGL turns X and Y coordinates between `-1.0` and `1.0` into surface
    /// coordinates, it will also map your Z coordinates to a certain range which you can
    /// specify here.
    ///
    /// The two values must be between `0.0` and `1.0`, anything outside this range will result
    /// in a panic. By default the depth range is `(0.0, 1.0)`.
    ///
    /// The first value of the tuple must be the "near" value, where `-1.0` will be mapped.
    /// The second value must be the "far" value, where `1.0` will be mapped.
    /// It is possible for the "near" value to be greater than the "far" value.
    pub range: (f32, f32),

    /// Sets whether the depth values of samples should be clamped to `0.0` and `1.0`.
    ///
    /// The default value is `NoClamp`.
    pub clamp: DepthClamp,
}

impl Default for Depth {
    #[inline]
    fn default() -> Depth {
        Depth {
            test: DepthTest::Overwrite,
            write: false,
            range: (0.0, 1.0),
            clamp: DepthClamp::NoClamp,
        }
    }
}

/// The function that the GPU will use to determine whether to write over an existing pixel
/// on the target.
///
/// # Depth buffers
///
/// After the fragment shader has been run, the GPU maps the output Z coordinates to the depth
/// range (which you can specify in the draw parameters) in order to obtain the depth value in
/// in window coordinates. This depth value is always between `0.0` and `1.0`.
///
/// In addition to the buffer where pixel colors are stored, you can also have a buffer
/// which contains the depth value of each pixel. Whenever the GPU tries to write a pixel,
/// it will first compare the depth value of the pixel to be written with the depth value that
/// is stored at this location. If `depth_write` is set to `true` in the draw parameters, it will
/// then write the depth value in the buffer.
///
/// The most common value for depth testing is to set `depth_test` to `IfLess`, and `depth_write`
/// to `true`.
///
/// If you don't have a depth buffer available, you can only pass `Overwrite`. Glium detects if
/// you pass any other value and reports an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthTest {
    /// Never replace the target pixel.
    ///
    /// This option doesn't really make sense, but is here for completeness.
    Ignore,

    /// Always replace the target pixel.
    ///
    /// This is the default mode.
    Overwrite,

    /// Replace if the z-value of the source is equal to the destination.
    IfEqual,

    /// Replace if the z-value of the source is different than the destination.
    IfNotEqual,

    /// Replace if the z-value of the source is more than the destination.
    IfMore,

    /// Replace if the z-value of the source is more than, or equal to the destination.
    IfMoreOrEqual,

    /// Replace if the z-value of the source is less than the destination.
    IfLess,

    /// Replace if the z-value of the source is less than, or equal to the destination.
    IfLessOrEqual
}

impl DepthTest {
    /// Returns true if the function requires a depth buffer to be used.
    #[inline]
    pub fn requires_depth_buffer(&self) -> bool {
        match *self {
            DepthTest::Ignore => true,
            DepthTest::Overwrite => false,
            DepthTest::IfEqual => true,
            DepthTest::IfNotEqual => true,
            DepthTest::IfMore => true,
            DepthTest::IfMoreOrEqual => true,
            DepthTest::IfLess => true,
            DepthTest::IfLessOrEqual => true,
        }
    }

    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            DepthTest::Ignore => gl::NEVER,
            DepthTest::Overwrite => gl::ALWAYS,
            DepthTest::IfEqual => gl::EQUAL,
            DepthTest::IfNotEqual => gl::NOTEQUAL,
            DepthTest::IfMore => gl::GREATER,
            DepthTest::IfMoreOrEqual => gl::GEQUAL,
            DepthTest::IfLess => gl::LESS,
            DepthTest::IfLessOrEqual => gl::LEQUAL,
        }
    }
}

/// Specifies whether the depth value of samples should be clamped to `0.0` or `1.0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthClamp {
    /// Do not clamp. Samples with values outside of the `[0.0, 1.0]` range will be discarded.
    ///
    /// This is the default value and is supported everywhere.
    NoClamp,

    /// Clamp the depth values. All samples will always be drawn.
    ///
    /// This value is only supported on OpenGL.
    Clamp,

    /// Depth values inferior to `0.0` will be clamped to `0.0`.
    ///
    /// **This option is supported only by very few OpenGL devices**.
    ClampNear,

    /// Depth values superior to `1.0` will be clamped to `1.0`.
    ///
    /// **This option is supported only by very few OpenGL devices**.
    ClampFar,
}

pub fn sync_depth(ctxt: &mut CommandContext, depth: &Depth) -> Result<(), DrawError> {
    // depth clamp
    {
        let state = &mut *ctxt.state;
        match (depth.clamp, &mut state.enabled_depth_clamp_near,
               &mut state.enabled_depth_clamp_far)
        {
            (DepthClamp::NoClamp, &mut false, &mut false) => (),
            (DepthClamp::Clamp, &mut true, &mut true) => (),

            (DepthClamp::NoClamp, near, far) => {
                if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_depth_clamp ||
                   ctxt.extensions.gl_nv_depth_clamp
                {
                    unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP) };
                    *near = false;
                    *far = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::Clamp, near, far) => {
                if ctxt.version >= &Version(Api::Gl, 3, 0) || ctxt.extensions.gl_arb_depth_clamp ||
                   ctxt.extensions.gl_nv_depth_clamp
                {
                    unsafe { ctxt.gl.Enable(gl::DEPTH_CLAMP) };
                    *near = true;
                    *far = true;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::ClampNear, &mut true, &mut false) => (),
            (DepthClamp::ClampFar, &mut false, &mut true) => (),

            (DepthClamp::ClampNear, &mut true, far) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_FAR_AMD) };
                    *far = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }

            },

            (DepthClamp::ClampNear, near @ &mut false, far) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Enable(gl::DEPTH_CLAMP_NEAR_AMD) };
                    if *far { unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_FAR_AMD); } }
                    *near = true;
                    *far = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::ClampFar, near, &mut true) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_NEAR_AMD) };
                    *near = false;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },

            (DepthClamp::ClampFar, near, far @ &mut false) => {
                if ctxt.extensions.gl_amd_depth_clamp_separate {
                    unsafe { ctxt.gl.Enable(gl::DEPTH_CLAMP_FAR_AMD) };
                    if *near { unsafe { ctxt.gl.Disable(gl::DEPTH_CLAMP_NEAR_AMD); } }
                    *near = false;
                    *far = true;
                } else {
                    return Err(DrawError::DepthClampNotSupported);
                }
            },
        }
    }

    // depth range
    if depth.range.0 < 0.0 || depth.range.0 > 1.0 ||
       depth.range.1 < 0.0 || depth.range.1 > 1.0
    {
        return Err(DrawError::InvalidDepthRange);
    }

    if depth.range != ctxt.state.depth_range {
        // TODO: WebGL requires depth.range.1 > depth.range.0
        unsafe {
            ctxt.gl.DepthRange(depth.range.0 as f64, depth.range.1 as f64);
        }
        ctxt.state.depth_range = depth.range;
    }

    if depth.test == DepthTest::Overwrite && !depth.write {
        // simply disabling GL_DEPTH_TEST
        if ctxt.state.enabled_depth_test {
            unsafe { ctxt.gl.Disable(gl::DEPTH_TEST) };
            ctxt.state.enabled_depth_test = false;
        }
        return Ok(());

    } else {
        if !ctxt.state.enabled_depth_test {
            unsafe { ctxt.gl.Enable(gl::DEPTH_TEST) };
            ctxt.state.enabled_depth_test = true;
        }
    }

    // depth test
    unsafe {
        let depth_test = depth.test.to_glenum();
        if ctxt.state.depth_func != depth_test {
            ctxt.gl.DepthFunc(depth_test);
            ctxt.state.depth_func = depth_test;
        }
    }

    // depth mask
    if depth.write != ctxt.state.depth_mask {
        unsafe {
            ctxt.gl.DepthMask(if depth.write { gl::TRUE } else { gl::FALSE });
        }
        ctxt.state.depth_mask = depth.write;
    }

    Ok(())
}
