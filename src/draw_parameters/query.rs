use backend::Facade;
use context::Context;
use context::CommandContext;
use ContextExt;
use ToGlEnum;
use GlObject;
use QueryExt;

use std::cell::Cell;
use std::fmt;
use std::mem;
use std::rc::Rc;

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

impl RawQuery {
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    pub fn new_if_supported<F>(facade: &F, ty: QueryType) -> Option<RawQuery> where F: Facade {
        let context = facade.get_context().clone();
        let ctxt = facade.get_context().make_current();

        // FIXME: handle Timestamp separately

        let id = unsafe {
            let mut id = mem::uninitialized();

            if ctxt.version >= &Version(Api::Gl, 3, 3) {
                match ty {
                    QueryType::AnySamplesPassed | QueryType::SamplesPassed |
                    QueryType::PrimitivesGenerated | QueryType::TimeElapsed |
                    QueryType::TransformFeedbackPrimitivesWritten => (),
                    QueryType::AnySamplesPassedConservative if
                            ctxt.extensions.gl_arb_es3_compatibility ||
                            ctxt.version >= &Version(Api:: Gl, 4, 3) => (),
                    _ => return None
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

                    _ => return None
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
                    _ => return None
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
                    _ => return None
                };

                ctxt.gl.GenQueries(1, &mut id);

            } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
                match ty {
                    QueryType::AnySamplesPassed | QueryType::AnySamplesPassedConservative => (),
                    _ => return None
                };

                ctxt.gl.GenQueriesEXT(1, &mut id);

            } else {
                return None;
            }

            id
        };

        Some(RawQuery {
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

        unsafe {
            let mut value = mem::uninitialized();

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

        unsafe {
            let mut value = mem::uninitialized();

            if ctxt.version >= &Version(Api::Gl, 1, 5) ||
               ctxt.version >= &Version(Api::GlEs, 3, 0)
            {
                ctxt.gl.GetQueryObjectuiv(self.id, gl::QUERY_RESULT, &mut value);

            } else if ctxt.extensions.gl_arb_occlusion_query {
                ctxt.gl.GetQueryObjectuivARB(self.id, gl::QUERY_RESULT, &mut value);

            } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
                ctxt.gl.GetQueryObjectuivEXT(self.id, gl::QUERY_RESULT, &mut value);

            } else {
                // if we reach here, user shouldn't have been able to create a query in the
                // first place
                unreachable!();
            }
            
            value
        }
    }

    /// Returns the value of the query. Blocks until it is available.
    ///
    /// This function doesn't block if `is_ready` returns true.
    pub fn get_u64(&self) -> u64 {
        let mut ctxt = self.context.make_current();
        self.deactivate(&mut ctxt);

        unsafe {
            if ctxt.version >= &Version(Api::Gl, 3, 3) {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectui64v(self.id, gl::QUERY_RESULT, &mut value);
                value

            } else if ctxt.version >= &Version(Api::Gl, 1, 5) ||
                      ctxt.version >= &Version(Api::GlEs, 3, 0)
            {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectuiv(self.id, gl::QUERY_RESULT, &mut value);
                value as u64

            } else if ctxt.extensions.gl_arb_occlusion_query {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectuivARB(self.id, gl::QUERY_RESULT, &mut value);
                value as u64

            } else if ctxt.extensions.gl_ext_occlusion_query_boolean {
                let mut value = mem::uninitialized();
                ctxt.gl.GetQueryObjectuivEXT(self.id, gl::QUERY_RESULT, &mut value);
                value as u64

            } else {
                // if we reach here, user shouldn't have been able to create a query in the
                // first place
                unreachable!();
            }
        }
    }

    /// Returns the value of the query. Blocks until it is available.
    ///
    /// This function doesn't block if `is_ready` returns true.
    pub fn get_bool(&self) -> bool {
        self.get_u32() != 0
    }

    /// If the query is active, unactivates it.
    fn deactivate(&self, ctxt: &mut CommandContext) {
        if ctxt.state.samples_passed_query == self.id {
            unsafe { ctxt.gl.EndQuery(gl::SAMPLES_PASSED) };
            ctxt.state.samples_passed_query = 0;
        }

        if ctxt.state.any_samples_passed_query == self.id {
            unsafe { ctxt.gl.EndQuery(gl::ANY_SAMPLES_PASSED) };
            ctxt.state.any_samples_passed_query = 0;
        }

        if ctxt.state.primitives_generated_query == self.id {
            unsafe { ctxt.gl.EndQuery(gl::PRIMITIVES_GENERATED) };
            ctxt.state.primitives_generated_query = 0;
        }

        if ctxt.state.transform_feedback_primitives_written_query == self.id {
            unsafe { ctxt.gl.EndQuery(gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN) };
            ctxt.state.transform_feedback_primitives_written_query = 0;
        }

        if ctxt.state.time_elapsed_query == self.id {
            unsafe { ctxt.gl.EndQuery(gl::TIME_ELAPSED) };
            ctxt.state.time_elapsed_query = 0;
        }
    }
}

impl Drop for RawQuery {
    fn drop(&mut self) {
        let mut ctxt = self.context.make_current();

        // FIXME: doesn't use ARB/EXT variants if necessary

        self.deactivate(&mut ctxt);

        unsafe {
            ctxt.gl.DeleteQueries(1, [self.id].as_ptr())
        }
    }
}

impl QueryExt for RawQuery {
    fn is_unused(&self) -> bool {
        !self.has_been_used.get()
    }

    fn set_used(&self) {
        self.has_been_used.set(true);
    }
}

impl fmt::Debug for RawQuery {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Query object #{}", self.id)
    }
}

impl GlObject for RawQuery {
    type Id = gl::types::GLuint;

    fn get_id(&self) -> gl::types::GLuint {
        self.id
    }
}

macro_rules! specialize {
    ($name:ident, $ty:expr, $ret:ty, $get_fn:ident) => {
        #[derive(Debug)]
        pub struct $name {
            query: RawQuery,
        }

        impl $name {
            /// Builds a new query. Returns `None` if the backend doesn't support this type.
            pub fn new_if_supported<F>(facade: &F) -> Option<$name> where F: Facade {
                RawQuery::new_if_supported(facade, $ty).map(|q| $name { query: q })
            }

            /// Queries the counter to see if the result is already available.
            pub fn is_ready(&self) -> bool {
                self.query.is_ready()
            }

            /// Returns the value of the query. Blocks until it is available.
            ///
            /// This function doesn't block if `is_ready` would return true.
            ///
            /// Note that you are strongly discouraged from calling this in the middle of the
            /// rendering process, as it may block for a long time.
            pub fn get(self) -> $ret {
                self.query.$get_fn()
            }
        }

        impl GlObject for $name {
            type Id = gl::types::GLuint;

            fn get_id(&self) -> gl::types::GLuint {
                self.query.get_id()
            }
        }

        impl QueryExt for $name {
            fn is_unused(&self) -> bool {
                self.query.is_unused()
            }

            fn set_used(&self) {
                self.query.set_used()
            }
        }
    };
}

specialize!(SamplesPassedQuery, QueryType::SamplesPassed, u32, get_u32);
specialize!(TimeElapsedQuery, QueryType::TimeElapsed, u32, get_u32);
