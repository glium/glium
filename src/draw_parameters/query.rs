use backend::Facade;
use context::Context;
use context::CommandContext;
use ContextExt;
use DrawError;
use ToGlEnum;
use GlObject;
use QueryExt;

use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
use std::error::Error;

use buffer::Buffer;
use buffer::BufferSlice;
use BufferExt;
use BufferSliceExt;

use gl;
use version::Api;
use version::Version;

pub struct RawQuery {
    context: Rc<Context>,
    id: gl::types::GLuint,
    ty: QueryType,

    // true means that this query has already been used or is being used to get data
    // this is important to know because we want to avoid erasing data
    has_been_used: Cell<bool>,
}

pub enum QueryType {
    SamplesPassed,
    AnySamplesPassed,
    AnySamplesPassedConservative,
    TimeElapsed,
    Timestamp,
    PrimitivesGenerated,
    TransformFeedbackPrimitivesWritten,
}

impl ToGlEnum for QueryType {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match *self {
            QueryType::SamplesPassed => gl::SAMPLES_PASSED,
            QueryType::AnySamplesPassed => gl::ANY_SAMPLES_PASSED,
            QueryType::AnySamplesPassedConservative => gl::ANY_SAMPLES_PASSED_CONSERVATIVE,
            QueryType::TimeElapsed => gl::TIME_ELAPSED,
            QueryType::Timestamp => gl::TIMESTAMP,
            QueryType::PrimitivesGenerated => gl::PRIMITIVES_GENERATED,
            QueryType::TransformFeedbackPrimitivesWritten => {
                gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN
            },
        }
    }
}

/// Error that can happen when creating a query object.
#[derive(Copy, Clone, Debug)]
pub enum QueryCreationError {
    /// The given query type is not supported.
    NotSupported,
}

impl fmt::Display for QueryCreationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for QueryCreationError {
    fn description(&self) -> &str {
        use self::QueryCreationError::*;
        match *self {
            NotSupported => "The given query type is not supported",
        }
    }
}

/// Error that can happen when writing the value of a query to a buffer.
#[derive(Copy, Clone, Debug)]
pub enum ToBufferError {
    /// Writing the result to a buffer is not supported.
    NotSupported,
}

impl fmt::Display for ToBufferError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error for ToBufferError {
    fn description(&self) -> &str {
        use self::ToBufferError::*;
        match *self {
            NotSupported => "Writing the result to a buffer is not supported",
        }
    }
}

impl RawQuery {
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    pub fn new<F: ?Sized>(facade: &F, ty: QueryType) -> Result<RawQuery, QueryCreationError>
                  where F: Facade
    {
        let context = facade.get_context().clone();
        let ctxt = facade.get_context().make_current();

        // FIXME: handle Timestamp separately

        let id = unsafe {
            let mut id = 0;

            if ctxt.version >= &Version(Api::Gl, 3, 3) {
                match ty {
                    QueryType::AnySamplesPassed | QueryType::SamplesPassed |
                    QueryType::PrimitivesGenerated | QueryType::TimeElapsed |
                    QueryType::TransformFeedbackPrimitivesWritten => (),
                    QueryType::AnySamplesPassedConservative if
                            ctxt.extensions.gl_arb_es3_compatibility ||
                            ctxt.version >= &Version(Api:: Gl, 4, 3) => (),
                    _ => return Err(QueryCreationError::NotSupported)
                };

                if ctxt.version >= &Version(Api:: Gl, 4, 5) ||
                   ctxt.extensions.gl_arb_direct_state_access
                {
                    ctxt.gl.CreateQueries(ty.to_glenum(), 1, &mut id);
                } else {
                    ctxt.gl.GenQueries(1, &mut id);
                }

            } else if ctxt.version >= &Version(Api::Gl, 3, 0) {
                match ty {
                    QueryType::SamplesPassed | QueryType::PrimitivesGenerated |
                    QueryType::TransformFeedbackPrimitivesWritten => (),
                    QueryType::AnySamplesPassed if ctxt.extensions.gl_arb_occlusion_query2 => (),
                    QueryType::AnySamplesPassedConservative if ctxt.extensions.gl_arb_es3_compatibility => (),
                    QueryType::TimeElapsed if ctxt.extensions.gl_arb_timer_query => (),

                    _ => return Err(QueryCreationError::NotSupported)
                };

                ctxt.gl.GenQueries(1, &mut id);

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) || ctxt.extensions.gl_arb_occlusion_query {
                match ty {
                    QueryType::SamplesPassed => (),
                    QueryType::AnySamplesPassed if ctxt.extensions.gl_arb_occlusion_query2 => (),
                    QueryType::AnySamplesPassedConservative if ctxt.extensions.gl_arb_es3_compatibility => (),
                    QueryType::PrimitivesGenerated if ctxt.extensions.gl_ext_transform_feedback => (),
                    QueryType::TransformFeedbackPrimitivesWritten if ctxt.extensions.gl_ext_transform_feedback => (),
                    QueryType::TimeElapsed if ctxt.extensions.gl_arb_timer_query => (),
                    _ => return Err(QueryCreationError::NotSupported)
                };

                if ctxt.version >= &Version(Api::Gl, 1, 5) {
                    ctxt.gl.GenQueries(1, &mut id);
                } else if ctxt.extensions.gl_arb_occlusion_query {
                    ctxt.gl.GenQueriesARB(1, &mut id);
                } else {
                    unreachable!();
                }

            } else if ctxt.version >= &Version(Api::GlEs, 3, 0) {
                match ty {
                    QueryType::AnySamplesPassed | QueryType::AnySamplesPassedConservative |
                    QueryType::TransformFeedbackPrimitivesWritten => (),
                    _ => return Err(QueryCreationError::NotSupported)
                };

                ctxt.gl.GenQueries(1, &mut id);

            } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
                match ty {
                    QueryType::AnySamplesPassed | QueryType::AnySamplesPassedConservative => (),
                    _ => return Err(QueryCreationError::NotSupported)
                };

                ctxt.gl.GenQueriesEXT(1, &mut id);

            } else {
                return Err(QueryCreationError::NotSupported);
            }

            id
        };

        Ok(RawQuery {
            context: context,
            id: id,
            ty: ty,
            has_been_used: Cell::new(false),
        })
    }

    /// Queries the counter to see if the result is already available.
    pub fn is_ready(&self) -> bool {
        let mut ctxt = self.context.make_current();
        self.deactivate(&mut ctxt);

        if !self.has_been_used.get() {
            return false;
        }

        Buffer::<u8>::unbind_query(&mut ctxt);

        unsafe {
            let mut value = 0;

            if ctxt.version >= &Version(Api::Gl, 1, 5) ||
               ctxt.version >= &Version(Api::GlEs, 3, 0)
            {
                ctxt.gl.GetQueryObjectuiv(self.id, gl::QUERY_RESULT_AVAILABLE, &mut value);

            } else if ctxt.extensions.gl_arb_occlusion_query {
                ctxt.gl.GetQueryObjectuivARB(self.id, gl::QUERY_RESULT_AVAILABLE, &mut value);

            } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
                ctxt.gl.GetQueryObjectuivEXT(self.id, gl::QUERY_RESULT_AVAILABLE, &mut value);

            } else {
                // if we reach here, user shouldn't have been able to create a query in the
                // first place
                unreachable!();
            }

            value != 0
        }
    }

    /// Returns the value of the query. Blocks until it is available.
    ///
    /// This function doesn't block if `is_ready` returns true.
    pub fn get_u32(&self) -> u32 {
        let mut ctxt = self.context.make_current();
        self.deactivate(&mut ctxt);

        if !self.has_been_used.get() {
            return 0;
        }

        Buffer::<u8>::unbind_query(&mut ctxt);

        unsafe {
            let mut value = 0;
            self.raw_get_u32(&mut ctxt, &mut value);
            value
        }
    }

    /// Writes the value of the query to a buffer.
    pub fn write_u32_to_buffer(&self, target: BufferSlice<u32>) -> Result<(), ToBufferError> {
        let mut ctxt = self.context.make_current();

        if !(ctxt.version >= &Version(Api::Gl, 4, 4) || ctxt.extensions.gl_arb_query_buffer_object ||
             ctxt.extensions.gl_amd_query_buffer_object)
        {
            return Err(ToBufferError::NotSupported);
        }

        self.deactivate(&mut ctxt);

        if !self.has_been_used.get() {
            panic!();
        }

        assert!(target.get_offset_bytes() % 4 == 0);

        target.prepare_and_bind_for_query(&mut ctxt);
        unsafe { self.raw_get_u32(&mut ctxt, target.get_offset_bytes() as *mut _); }

        if let Some(fence) = target.add_fence() {
            fence.insert(&mut ctxt);
        }

        Ok(())
    }

    unsafe fn raw_get_u32(&self, ctxt: &mut CommandContext, target: *mut gl::types::GLuint) {
        if ctxt.version >= &Version(Api::Gl, 1, 5) || ctxt.version >= &Version(Api::GlEs, 3, 0) {
            ctxt.gl.GetQueryObjectuiv(self.id, gl::QUERY_RESULT, target);

        } else if ctxt.extensions.gl_arb_occlusion_query {
            ctxt.gl.GetQueryObjectuivARB(self.id, gl::QUERY_RESULT, target);

        } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
            ctxt.gl.GetQueryObjectuivEXT(self.id, gl::QUERY_RESULT, target);

        } else {
            // if we reach here, user shouldn't have been able to create a query in the
            // first place
            unreachable!();
        }
    }

    /// Returns the value of the query. Blocks until it is available.
    ///
    /// This function doesn't block if `is_ready` returns true.
    pub fn get_u64(&self) -> u64 {
        let mut ctxt = self.context.make_current();
        self.deactivate(&mut ctxt);

        if !self.has_been_used.get() {
            return 0;
        }

        Buffer::<u8>::unbind_query(&mut ctxt);

        unsafe {
            let mut value = 0;
            if let Ok(_) = self.raw_get_u64(&mut ctxt, &mut value) {
                return value;
            }

            let mut value = 0;
            self.raw_get_u32(&mut ctxt, &mut value);
            value as u64
        }
    }

    unsafe fn raw_get_u64(&self, ctxt: &mut CommandContext, target: *mut gl::types::GLuint64)
                          -> Result<(), ()>
    {
        if ctxt.version >= &Version(Api::Gl, 3, 3) {
            ctxt.gl.GetQueryObjectui64v(self.id, gl::QUERY_RESULT, target);
            Ok(())

        } else {
            Err(())
        }
    }

    /// Returns the value of the query. Blocks until it is available.
    ///
    /// This function doesn't block if `is_ready` returns true.
    #[inline]
    pub fn get_bool(&self) -> bool {
        self.get_u32() != 0
    }

    /// If the query is active, unactivates it.
    fn deactivate(&self, ctxt: &mut CommandContext) {
        if ctxt.state.samples_passed_query == self.id {
            unsafe { raw_end_query(ctxt, gl::SAMPLES_PASSED) };
            ctxt.state.samples_passed_query = 0;
        }

        if ctxt.state.any_samples_passed_query == self.id {
            unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED) };
            ctxt.state.any_samples_passed_query = 0;
        }

        if ctxt.state.any_samples_passed_conservative_query == self.id {
            unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED_CONSERVATIVE) };
            ctxt.state.any_samples_passed_conservative_query = 0;
        }

        if ctxt.state.primitives_generated_query == self.id {
            unsafe { raw_end_query(ctxt, gl::PRIMITIVES_GENERATED) };
            ctxt.state.primitives_generated_query = 0;
        }

        if ctxt.state.transform_feedback_primitives_written_query == self.id {
            unsafe { raw_end_query(ctxt, gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN) };
            ctxt.state.transform_feedback_primitives_written_query = 0;
        }

        if ctxt.state.time_elapsed_query == self.id {
            unsafe { raw_end_query(ctxt, gl::TIME_ELAPSED) };
            ctxt.state.time_elapsed_query = 0;
        }
    }
}

impl Drop for RawQuery {
    fn drop(&mut self) {
        let mut ctxt = self.context.make_current();
        self.deactivate(&mut ctxt);

        if let Some((id, _)) = ctxt.state.conditional_render {
            if id == self.id {
                RawQuery::end_conditional_render(&mut ctxt);
            }
        }

        unsafe {
            if ctxt.version >= &Version(Api::Gl, 1, 5) ||
               ctxt.version >= &Version(Api::GlEs, 3, 0)
            {
                ctxt.gl.DeleteQueries(1, [self.id].as_ptr());

            } else if ctxt.extensions.gl_arb_occlusion_query {
                ctxt.gl.DeleteQueriesARB(1, [self.id].as_ptr());

            } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
                ctxt.gl.DeleteQueriesEXT(1, [self.id].as_ptr());

            } else {
                unreachable!();
            }
        }
    }
}

impl QueryExt for RawQuery {
    fn begin_query(&self, ctxt: &mut CommandContext) -> Result<(), DrawError> {
        match self.ty {
            QueryType::SamplesPassed => {
                if ctxt.state.any_samples_passed_query != 0 {
                    ctxt.state.any_samples_passed_query = 0;
                    unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED); }
                }

                if ctxt.state.any_samples_passed_conservative_query != 0 {
                    ctxt.state.any_samples_passed_conservative_query = 0;
                    unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED_CONSERVATIVE); }
                }

                if ctxt.state.samples_passed_query != self.id {
                    if self.has_been_used.get() {
                        return Err(DrawError::WrongQueryOperation);
                    }

                    unsafe {
                        if ctxt.state.samples_passed_query != 0 {
                            raw_end_query(ctxt, gl::SAMPLES_PASSED);
                        }
                        raw_begin_query(ctxt, gl::SAMPLES_PASSED, self.id);
                    }

                    self.has_been_used.set(true);
                    ctxt.state.samples_passed_query = self.id;
                }
            },

            QueryType::AnySamplesPassed => {
                if ctxt.state.samples_passed_query != 0 {
                    ctxt.state.samples_passed_query = 0;
                    unsafe { raw_end_query(ctxt, gl::SAMPLES_PASSED); }
                }

                if ctxt.state.any_samples_passed_conservative_query != 0 {
                    ctxt.state.any_samples_passed_conservative_query = 0;
                    unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED_CONSERVATIVE); }
                }

                if ctxt.state.any_samples_passed_query != self.id {
                    if self.has_been_used.get() {
                        return Err(DrawError::WrongQueryOperation);
                    }

                    unsafe {
                        if ctxt.state.any_samples_passed_query != 0 {
                            raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED);
                        }
                        raw_begin_query(ctxt, gl::ANY_SAMPLES_PASSED, self.id);
                    }

                    self.has_been_used.set(true);
                    ctxt.state.any_samples_passed_query = self.id;
                }
            },

            QueryType::AnySamplesPassedConservative => {
                if ctxt.state.samples_passed_query != 0 {
                    ctxt.state.samples_passed_query = 0;
                    unsafe { raw_end_query(ctxt, gl::SAMPLES_PASSED); }
                }

                if ctxt.state.any_samples_passed_query != 0 {
                    ctxt.state.any_samples_passed_query = 0;
                    unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED); }
                }

                if ctxt.state.any_samples_passed_conservative_query != self.id {
                    if self.has_been_used.get() {
                        return Err(DrawError::WrongQueryOperation);
                    }

                    unsafe {
                        if ctxt.state.any_samples_passed_conservative_query != 0 {
                            raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED_CONSERVATIVE);
                        }
                        raw_begin_query(ctxt, gl::ANY_SAMPLES_PASSED_CONSERVATIVE, self.id);
                    }

                    self.has_been_used.set(true);
                    ctxt.state.any_samples_passed_conservative_query = self.id;
                }
            },

            QueryType::TimeElapsed => {
                if ctxt.state.time_elapsed_query != self.id {
                    if self.has_been_used.get() {
                        return Err(DrawError::WrongQueryOperation);
                    }

                    unsafe {
                        if ctxt.state.time_elapsed_query != 0 {
                            raw_end_query(ctxt, gl::TIME_ELAPSED);
                        }
                        raw_begin_query(ctxt, gl::TIME_ELAPSED, self.id);
                    }

                    self.has_been_used.set(true);
                    ctxt.state.time_elapsed_query = self.id;
                }
            },

            QueryType::Timestamp => panic!(),

            QueryType::PrimitivesGenerated => {
                if ctxt.state.primitives_generated_query != self.id {
                    if self.has_been_used.get() {
                        return Err(DrawError::WrongQueryOperation);
                    }

                    unsafe {
                        if ctxt.state.primitives_generated_query != 0 {
                            raw_end_query(ctxt, gl::PRIMITIVES_GENERATED);
                        }
                        raw_begin_query(ctxt, gl::PRIMITIVES_GENERATED, self.id);
                    }

                    self.has_been_used.set(true);
                    ctxt.state.primitives_generated_query = self.id;
                }
            },

            QueryType::TransformFeedbackPrimitivesWritten => {
                if ctxt.state.transform_feedback_primitives_written_query != self.id {
                    if self.has_been_used.get() {
                        return Err(DrawError::WrongQueryOperation);
                    }

                    unsafe {
                        if ctxt.state.transform_feedback_primitives_written_query != 0 {
                            raw_end_query(ctxt, gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN);
                        }
                        raw_begin_query(ctxt, gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, self.id);
                    }

                    self.has_been_used.set(true);
                    ctxt.state.transform_feedback_primitives_written_query = self.id;
                }
            },
        };

        Ok(())
    }

    fn end_samples_passed_query(ctxt: &mut CommandContext) {
        if ctxt.state.samples_passed_query != 0 {
            ctxt.state.samples_passed_query = 0;
            unsafe { raw_end_query(ctxt, gl::SAMPLES_PASSED); }
        }

        if ctxt.state.any_samples_passed_query != 0 {
            ctxt.state.any_samples_passed_query = 0;
            unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED); }
        }

        if ctxt.state.any_samples_passed_conservative_query != 0 {
            ctxt.state.any_samples_passed_conservative_query = 0;
            unsafe { raw_end_query(ctxt, gl::ANY_SAMPLES_PASSED_CONSERVATIVE); }
        }
    }

    #[inline]
    fn end_time_elapsed_query(ctxt: &mut CommandContext) {
        if ctxt.state.time_elapsed_query != 0 {
            ctxt.state.time_elapsed_query = 0;
            unsafe { raw_end_query(ctxt, gl::TIME_ELAPSED); }
        }
    }

    #[inline]
    fn end_primitives_generated_query(ctxt: &mut CommandContext) {
        if ctxt.state.primitives_generated_query != 0 {
            ctxt.state.primitives_generated_query = 0;
            unsafe { raw_end_query(ctxt, gl::PRIMITIVES_GENERATED); }
        }
    }

    #[inline]
    fn end_transform_feedback_primitives_written_query(ctxt: &mut CommandContext) {
        if ctxt.state.transform_feedback_primitives_written_query != 0 {
            ctxt.state.transform_feedback_primitives_written_query = 0;
            unsafe { raw_end_query(ctxt, gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN); }
        }
    }

    fn begin_conditional_render(&self, ctxt: &mut CommandContext, wait: bool, per_region: bool) {
        let new_mode = match (wait, per_region) {
            (true, true) => gl::QUERY_BY_REGION_WAIT,
            (true, false) => gl::QUERY_WAIT,
            (false, true) => gl::QUERY_BY_REGION_NO_WAIT,
            (false, false) => gl::QUERY_NO_WAIT,
        };

        // returning if the active conditional rendering is already good
        if let Some((old_id, old_mode)) = ctxt.state.conditional_render {
            if old_id == self.id {
                // if the new mode is "no_wait" but he old mode is "wait",
                // then we don't need to change it
                match (new_mode, old_mode) {
                    (a, b) if a == b => return,
                    (gl::QUERY_NO_WAIT, gl::QUERY_WAIT) => return,
                    (gl::QUERY_BY_REGION_NO_WAIT, gl::QUERY_BY_REGION_WAIT) => return,
                    _ => (),
                }
            }
        }

        // de-activating the existing conditional render first
        if ctxt.state.conditional_render.is_some() {
            RawQuery::end_conditional_render(ctxt);
        }

        // de-activating the query
        self.deactivate(ctxt);

        // activating
        if ctxt.version >= &Version(Api::Gl, 3, 0) {
            unsafe { ctxt.gl.BeginConditionalRender(self.id, new_mode) };
        } else if ctxt.extensions.gl_nv_conditional_render {
            unsafe { ctxt.gl.BeginConditionalRenderNV(self.id, new_mode) };
        } else {
            unreachable!();
        }

        ctxt.state.conditional_render = Some((self.id, new_mode));
    }

    fn end_conditional_render(ctxt: &mut CommandContext) {
        if ctxt.state.conditional_render.is_none() {
            return;
        }

        if ctxt.version >= &Version(Api::Gl, 3, 0) {
            unsafe { ctxt.gl.EndConditionalRender(); }
        } else if ctxt.extensions.gl_nv_conditional_render {
            unsafe { ctxt.gl.EndConditionalRenderNV(); }
        } else {
            unreachable!();
        }

        ctxt.state.conditional_render = None;
    }

    fn is_unused(&self) -> bool {
        !self.has_been_used.get()
    }
}

impl fmt::Debug for RawQuery {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Query object #{}", self.id)
    }
}

impl GlObject for RawQuery {
    type Id = gl::types::GLuint;

    #[inline]
    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

/// Calls `glBeginQuery`.
///
/// # Unsafe
///
/// The type of query must be guaranteed to be supported by the backend.
/// The id of the query must be valid.
///
unsafe fn raw_begin_query(ctxt: &mut CommandContext, ty: gl::types::GLenum, id: gl::types::GLuint) {
    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
       ctxt.version >= &Version(Api::GlEs, 3, 0)
    {
        ctxt.gl.BeginQuery(ty, id);

    } else if ctxt.extensions.gl_arb_occlusion_query {
        ctxt.gl.BeginQueryARB(ty, id);

    } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
        ctxt.gl.BeginQueryEXT(ty, id);

    } else {
        unreachable!();
    }
}

/// Calls `glEndQuery`.
///
/// # Unsafe
///
/// The type of query must be guaranteed to be supported by the backend.
unsafe fn raw_end_query(ctxt: &mut CommandContext, ty: gl::types::GLenum) {
    if ctxt.version >= &Version(Api::Gl, 1, 5) ||
       ctxt.version >= &Version(Api::GlEs, 3, 0)
    {
        ctxt.gl.EndQuery(ty);

    } else if ctxt.extensions.gl_arb_occlusion_query {
        ctxt.gl.EndQueryARB(ty);

    } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
        ctxt.gl.EndQueryEXT(ty);

    } else {
        unreachable!();
    }
}

macro_rules! impl_helper {
    ($name:ident, $ret:ty, $get_fn:ident) => {
        impl $name {
            /// Queries the counter to see if the result is already available.
            #[inline]
            pub fn is_ready(&self) -> bool {
                self.query.is_ready()
            }

            /// Returns the value of the query. Blocks until it is available.
            ///
            /// This function doesn't block if `is_ready` would return true.
            ///
            /// Note that you are strongly discouraged from calling this in the middle of the
            /// rendering process, as it may block for a long time.
            ///
            /// Queries should either have their result written into a buffer, be used for
            /// conditional rendering, or stored and checked during the next frame.
            #[inline]
            pub fn get(self) -> $ret {
                self.query.$get_fn()
            }

            /// Writes the result of the query to a buffer when it is available.
            ///
            /// This function doesn't block. Instead it submits a commands to the GPU's commands
            /// queue and orders the GPU to write the result of the query to a buffer.
            ///
            /// This operation is not necessarily supported everywhere.
            #[inline]
            pub fn to_buffer_u32(&self, target: BufferSlice<u32>)
                                 -> Result<(), ToBufferError>
            {
                self.query.write_u32_to_buffer(target)
            }
        }

        impl GlObject for $name {
            type Id = gl::types::GLuint;

            #[inline]
            fn get_id(&self) -> gl::types::GLuint {
                self.query.get_id()
            }
        }

        impl QueryExt for $name {
            #[inline]
            fn begin_query(&self, ctxt: &mut CommandContext) -> Result<(), DrawError> {
                self.query.begin_query(ctxt)
            }

            #[inline]
            fn end_samples_passed_query(ctxt: &mut CommandContext) {
                RawQuery::end_samples_passed_query(ctxt)
            }

            #[inline]
            fn end_time_elapsed_query(ctxt: &mut CommandContext) {
                RawQuery::end_time_elapsed_query(ctxt)
            }

            #[inline]
            fn end_primitives_generated_query(ctxt: &mut CommandContext) {
                RawQuery::end_primitives_generated_query(ctxt)
            }

            #[inline]
            fn end_transform_feedback_primitives_written_query(ctxt: &mut CommandContext) {
                RawQuery::end_transform_feedback_primitives_written_query(ctxt)
            }

            #[inline]
            fn begin_conditional_render(&self, ctxt: &mut CommandContext, wait: bool, per_region: bool) {
                self.query.begin_conditional_render(ctxt, wait, per_region)
            }

            #[inline]
            fn end_conditional_render(ctxt: &mut CommandContext) {
                RawQuery::end_conditional_render(ctxt)
            }

            #[inline]
            fn is_unused(&self) -> bool {
                self.query.is_unused()
            }
        }
    };
}

/// A query that allows you to know the number of samples written to the output during the
/// draw operations where this query was active.
///
/// If you just want to know whether or not some samples have been written, you should use
/// a `AnySamplesPassedQuery` query instead.
#[derive(Debug)]
pub struct SamplesPassedQuery {
    query: RawQuery,
}

impl SamplesPassedQuery {
    /// Builds a new query.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F) -> Result<SamplesPassedQuery, QueryCreationError> where F: Facade {
        RawQuery::new(facade, QueryType::SamplesPassed).map(|q| SamplesPassedQuery { query: q })
    }
}

impl_helper!(SamplesPassedQuery, u32, get_u32);

/// A query that allows you to know the number of nanoseconds that have elapsed
/// during the draw operations.
///
/// TODO: not sure that it's nanoseconds
#[derive(Debug)]
pub struct TimeElapsedQuery {
    query: RawQuery,
}

impl TimeElapsedQuery {
    /// Builds a new query.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F) -> Result<TimeElapsedQuery, QueryCreationError> where F: Facade {
        RawQuery::new(facade, QueryType::TimeElapsed).map(|q| TimeElapsedQuery { query: q })
    }
}

impl_helper!(TimeElapsedQuery, u32, get_u32);

/// A query type that allows you to know whether any sample has been written to the output during
/// the operations executed with this query.
///
/// ## OpenGL
///
/// This is usually a query of type `GL_ANY_SAMPLES_PASSED` or
/// `GL_ANY_SAMPLES_PASSED_CONSERVATIVE`.
///
/// However if the backend doesn't support conservative queries, glium will automatically fall
/// back to a non-conservative query. If the backend doesn't support either types but supports
/// `GL_SAMPLES_PASSED`, then glium will automatically use a `GL_SAMPLES_PASSED` query instead.
#[derive(Debug)]
pub struct AnySamplesPassedQuery {
    query: RawQuery,
}

impl AnySamplesPassedQuery {
    /// Builds a new query.
    ///
    /// If you pass `true` for `conservative`, then OpenGL may use a less accurate algorithm,
    /// leading to a faster result but with more false positives.
    pub fn new<F: ?Sized>(facade: &F, conservative: bool)
                  -> Result<AnySamplesPassedQuery, QueryCreationError>
                  where F: Facade
    {
        if conservative {
            if let Ok(q) = RawQuery::new(facade, QueryType::AnySamplesPassedConservative) {
                return Ok(AnySamplesPassedQuery { query: q });
            }
        }

        if let Ok(q) = RawQuery::new(facade, QueryType::AnySamplesPassed) {
            return Ok(AnySamplesPassedQuery { query: q });
        } else if let Ok(q) = RawQuery::new(facade, QueryType::SamplesPassed) {
            return Ok(AnySamplesPassedQuery { query: q });
        } else {
            return Err(QueryCreationError::NotSupported);
        }
    }
}

impl_helper!(AnySamplesPassedQuery, bool, get_bool);

/// Query that allows you to know the number of primitives generated by the geometry shader.
/// Will stay at `0` if you use it without any active geometry shader.
#[derive(Debug)]
pub struct PrimitivesGeneratedQuery {
    query: RawQuery,
}

impl PrimitivesGeneratedQuery {
    /// Builds a new query.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F) -> Result<PrimitivesGeneratedQuery, QueryCreationError>
                  where F: Facade
    {
        RawQuery::new(facade, QueryType::PrimitivesGenerated)
                                                    .map(|q| PrimitivesGeneratedQuery { query: q })
    }
}

impl_helper!(PrimitivesGeneratedQuery, u32, get_u32);

/// Query that allows you to know the number of primitives generated by transform feedback.
#[derive(Debug)]
pub struct TransformFeedbackPrimitivesWrittenQuery {
    query: RawQuery,
}

impl TransformFeedbackPrimitivesWrittenQuery {
    /// Builds a new query.
    #[inline]
    pub fn new<F: ?Sized>(facade: &F) -> Result<TransformFeedbackPrimitivesWrittenQuery, QueryCreationError>
                  where F: Facade
    {
        RawQuery::new(facade, QueryType::TransformFeedbackPrimitivesWritten)
                                     .map(|q| TransformFeedbackPrimitivesWrittenQuery { query: q })
    }
}

impl_helper!(TransformFeedbackPrimitivesWrittenQuery, u32, get_u32);
