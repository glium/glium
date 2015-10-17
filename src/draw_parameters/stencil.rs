use gl;
use context;
use ToGlEnum;

#[derive(Copy, Clone, Debug)]
pub struct Stencil {
    /// A comparison against the existing value in the stencil buffer.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_test_counter_clockwise` instead.
    ///
    /// The default value is `AlwaysPass`.
    pub test_clockwise: StencilTest,

    /// Reference value that is used by `stencil_test_clockwise`, `stencil_fail_operation_clockwise`,
    /// `stencil_pass_depth_fail_operation_clockwise` and `stencil_depth_pass_operation_clockwise`.
    pub reference_value_clockwise: i32,

    /// Allows specifying a mask when writing data on the stencil buffer.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_write_mask_counter_clockwise` instead.
    ///
    /// The default value is `0xffffffff`.
    pub write_mask_clockwise: u32,

    /// Specifies the operation to do when a fragment fails the stencil test.
    ///
    /// The stencil test is the test specified by `stencil_test_clockwise`.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_fail_operation_counter_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub fail_operation_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes the stencil test but fails
    /// the depth test.
    ///
    /// The stencil test is the test specified by `stencil_test_clockwise`.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_pass_depth_fail_operation_counter_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub pass_depth_fail_operation_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes both the stencil and depth tests.
    ///
    /// The stencil test is the test specified by `stencil_test_clockwise`.
    ///
    /// Only relevant for faces that are clockwise on the target surface. Other faces, points and
    /// lines use `stencil_depth_pass_operation_counter_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub depth_pass_operation_clockwise: StencilOperation,

    /// A comparaison against the existing value in the stencil buffer.
    ///
    /// Only relevant for points, lines and faces that are counter-clockwise on the target surface.
    /// Other faces use `stencil_test_counter_clockwise` instead.
    ///
    /// The default value is `AlwaysPass`.
    pub test_counter_clockwise: StencilTest,

    /// Reference value that is used by `stencil_test_counter_clockwise`,
    /// `stencil_fail_operation_counter_clockwise`,
    /// `stencil_pass_depth_fail_operation_counter_clockwise` and
    /// `stencil_depth_pass_operation_counter_clockwise`.
    pub reference_value_counter_clockwise: i32,

    /// Allows specifying a mask when writing data on the stencil buffer.
    ///
    /// Only relevant for points, lines and faces that are counter-clockwise on the target surface.
    /// Other faces use `stencil_write_mask_clockwise` instead.
    ///
    /// The default value is `0xffffffff`.
    pub write_mask_counter_clockwise: u32,

    /// Specifies the operation to do when a fragment fails the stencil test.
    ///
    /// The stencil test is the test specified by `stencil_test_counter_clockwise`.
    ///
    /// Only relevant for faces that are counter-clockwise on the target surface. Other faces
    /// use `stencil_fail_operation_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub fail_operation_counter_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes the stencil test but fails
    /// the depth test.
    ///
    /// The stencil test is the test specified by `stencil_test_counter_clockwise`.
    ///
    /// Only relevant for faces that are counter-clockwise on the target surface. Other faces
    /// use `stencil_pass_depth_fail_operation_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub pass_depth_fail_operation_counter_clockwise: StencilOperation,

    /// Specifies the operation to do when a fragment passes both the stencil and depth tests.
    ///
    /// The stencil test is the test specified by `stencil_test_counter_clockwise`.
    ///
    /// Only relevant for faces that are counter-clockwise on the target surface. Other faces
    /// use `stencil_depth_pass_operation_clockwise` instead.
    ///
    /// The default value is `Keep`.
    pub depth_pass_operation_counter_clockwise: StencilOperation,
}

impl Default for Stencil {
    #[inline]
    fn default() -> Stencil {
        Stencil {
            test_clockwise: StencilTest::AlwaysPass,
            reference_value_clockwise: 0,
            write_mask_clockwise: 0xffffffff,
            fail_operation_clockwise: StencilOperation::Keep,
            pass_depth_fail_operation_clockwise: StencilOperation::Keep,
            depth_pass_operation_clockwise: StencilOperation::Keep,
            test_counter_clockwise: StencilTest::AlwaysPass,
            reference_value_counter_clockwise: 0,
            write_mask_counter_clockwise: 0xffffffff,
            fail_operation_counter_clockwise: StencilOperation::Keep,
            pass_depth_fail_operation_counter_clockwise: StencilOperation::Keep,
            depth_pass_operation_counter_clockwise: StencilOperation::Keep,
        }
    }
}

/// Specifies which comparison the GPU will do to determine whether a sample passes the stencil
/// test. The general equation is `(ref & mask) CMP (stencil & mask)`, where `ref` is the reference
/// value (`stencil_reference_value_clockwise` or `stencil_reference_value_counter_clockwise`),
/// `CMP` is the comparison chosen, and `stencil` is the current value in the stencil buffer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StencilTest {
    /// The stencil test always passes.
    AlwaysPass,

    /// The stencil test always fails.
    AlwaysFail,

    /// `(ref & mask) < (stencil & mask)`
    IfLess {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) <= (stencil & mask)`
    IfLessOrEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) > (stencil & mask)`
    IfMore {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) >= (stencil & mask)`
    IfMoreOrEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) == (stencil & mask)`
    IfEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },

    /// `(ref & mask) != (stencil & mask)`
    IfNotEqual {
        /// The mask that is and'ed with the reference value and stencil buffer.
        mask: u32,
    },
}

/// Specificies which operation the GPU will do depending on the result of the stencil test.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StencilOperation {
    /// Keeps the value currently in the stencil buffer.
    Keep,

    /// Writes zero in the stencil buffer.
    Zero,

    /// Writes the reference value (`stencil_reference_value_clockwise` or
    /// `stencil_reference_value_counter_clockwise`) in the stencil buffer.
    Replace,

    /// Increments the value currently in the stencil buffer. If the value is the
    /// maximum, don't do anything.
    Increment,

    /// Increments the value currently in the stencil buffer. If the value is the
    /// maximum, wrap to `0`.
    IncrementWrap,

    /// Decrements the value currently in the stencil buffer. If the value is `0`,
    /// don't do anything.
    Decrement,

    /// Decrements the value currently in the stencil buffer. If the value is `0`,
    /// wrap to `-1`.
    DecrementWrap,

    /// Inverts each bit of the value.
    Invert,
}

impl ToGlEnum for StencilOperation {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            StencilOperation::Keep => gl::KEEP,
            StencilOperation::Zero => gl::ZERO,
            StencilOperation::Replace => gl::REPLACE,
            StencilOperation::Increment => gl::INCR,
            StencilOperation::IncrementWrap => gl::INCR_WRAP,
            StencilOperation::Decrement => gl::DECR,
            StencilOperation::DecrementWrap => gl::DECR_WRAP,
            StencilOperation::Invert => gl::INVERT,
        }
    }
}

pub fn sync_stencil(ctxt: &mut context::CommandContext, params: &Stencil) {
    // TODO: optimize me

    let (test_cw, read_mask_cw) = match params.test_clockwise {
        StencilTest::AlwaysPass => (gl::ALWAYS, 0),
        StencilTest::AlwaysFail => (gl::NEVER, 0),
        StencilTest::IfLess { mask } => (gl::LESS, mask),
        StencilTest::IfLessOrEqual { mask } => (gl::LEQUAL, mask),
        StencilTest::IfMore { mask } => (gl::GREATER, mask),
        StencilTest::IfMoreOrEqual { mask } => (gl::GEQUAL, mask),
        StencilTest::IfEqual { mask } => (gl::EQUAL, mask),
        StencilTest::IfNotEqual { mask } => (gl::NOTEQUAL, mask),
    };

    let (test_ccw, read_mask_ccw) = match params.test_counter_clockwise {
        StencilTest::AlwaysPass => (gl::ALWAYS, 0),
        StencilTest::AlwaysFail => (gl::NEVER, 0),
        StencilTest::IfLess { mask } => (gl::LESS, mask),
        StencilTest::IfLessOrEqual { mask } => (gl::LEQUAL, mask),
        StencilTest::IfMore { mask } => (gl::GREATER, mask),
        StencilTest::IfMoreOrEqual { mask } => (gl::GEQUAL, mask),
        StencilTest::IfEqual { mask } => (gl::EQUAL, mask),
        StencilTest::IfNotEqual { mask } => (gl::NOTEQUAL, mask),
    };

    if ctxt.state.stencil_func_back != (test_cw, params.reference_value_clockwise, read_mask_cw) {
        unsafe {
            ctxt.gl.StencilFuncSeparate(gl::BACK,
                                        test_cw,
                                        params.reference_value_clockwise,
                                        read_mask_cw)
        }
        ctxt.state.stencil_func_back = (test_cw, params.reference_value_clockwise, read_mask_cw);
    }

    if ctxt.state.stencil_func_front !=
       (test_ccw,
        params.reference_value_counter_clockwise,
        read_mask_ccw) {
        unsafe {
            ctxt.gl.StencilFuncSeparate(gl::FRONT,
                                        test_ccw,
                                        params.reference_value_counter_clockwise,
                                        read_mask_ccw)
        }
        ctxt.state.stencil_func_front = (test_ccw,
                                         params.reference_value_counter_clockwise,
                                         read_mask_ccw);
    }

    if ctxt.state.stencil_mask_back != params.write_mask_clockwise {
        unsafe { ctxt.gl.StencilMaskSeparate(gl::BACK, params.write_mask_clockwise) }
        ctxt.state.stencil_mask_back = params.write_mask_clockwise;
    }

    if ctxt.state.stencil_mask_front != params.write_mask_clockwise {
        unsafe { ctxt.gl.StencilMaskSeparate(gl::FRONT, params.write_mask_clockwise) }
        ctxt.state.stencil_mask_front = params.write_mask_clockwise;
    }

    let op_back = (params.fail_operation_clockwise.to_glenum(),
                   params.pass_depth_fail_operation_clockwise.to_glenum(),
                   params.depth_pass_operation_clockwise.to_glenum());
    if ctxt.state.stencil_op_back != op_back {
        unsafe { ctxt.gl.StencilOpSeparate(gl::BACK, op_back.0, op_back.1, op_back.2) }
        ctxt.state.stencil_op_back = op_back;
    }

    let op_front = (params.fail_operation_counter_clockwise.to_glenum(),
                    params.pass_depth_fail_operation_counter_clockwise.to_glenum(),
                    params.depth_pass_operation_counter_clockwise.to_glenum());
    if ctxt.state.stencil_op_front != op_front {
        unsafe { ctxt.gl.StencilOpSeparate(gl::FRONT, op_front.0, op_front.1, op_front.2) }
        ctxt.state.stencil_op_front = op_front;
    }

    let enable_stencil = test_cw != gl::ALWAYS || test_ccw != gl::ALWAYS ||
                         op_back.0 != gl::KEEP || op_front.0 != gl::KEEP ||
                         op_back.1 != gl::KEEP || op_front.1 != gl::KEEP ||
                         op_back.2 != gl::KEEP || op_front.2 != gl::KEEP;
    if ctxt.state.enabled_stencil_test != enable_stencil {
        if enable_stencil {
            unsafe { ctxt.gl.Enable(gl::STENCIL_TEST) }
        } else {
            unsafe { ctxt.gl.Disable(gl::STENCIL_TEST) }
        }

        ctxt.state.enabled_stencil_test = enable_stencil;
    }
}
