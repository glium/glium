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

        if ctxt.state.any_samples_passed_conservative_query == self.id {
            unsafe { ctxt.gl.EndQuery(gl::ANY_SAMPLES_PASSED_CONSERVATIVE) };
            ctxt.state.any_samples_passed_conservative_query = 0;
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

        if let Some((id, _)) = ctxt.state.conditional_render {
            if id == self.id {
                if ctxt.version >= &Version(Api::Gl, 3, 0) {
                    unsafe { ctxt.gl.EndConditionalRender() };
                } else if ctxt.extensions.gl_nv_conditional_render {
                    unsafe { ctxt.gl.EndConditionalRenderNV() };
                } else {
                    unreachable!();
                }
            }
        }

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

    fn get_type(&self) -> gl::types::GLenum {
        self.ty.to_glenum()
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

macro_rules! impl_helper {
    ($name:ident, $ret:ty, $get_fn:ident) => {
        impl $name {
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
            ///
            /// Queries should either have their result written into a buffer, be used for
            /// conditional rendering, or stored and checked during the next frame.
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

            fn get_type(&self) -> gl::types::GLenum {
                self.query.get_type()
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
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    pub fn new_if_supported<F>(facade: &F) -> Option<SamplesPassedQuery> where F: Facade {
        RawQuery::new_if_supported(facade, QueryType::SamplesPassed)
                .map(|q| SamplesPassedQuery { query: q })
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
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    pub fn new_if_supported<F>(facade: &F) -> Option<TimeElapsedQuery> where F: Facade {
        RawQuery::new_if_supported(facade, QueryType::TimeElapsed)
                .map(|q| TimeElapsedQuery { query: q })
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
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    ///
    /// If you pass `true` for `conservative`, then OpenGL may use a less accurate algorithm,
    /// leading to a faster result but with more false positives.
    pub fn new_if_supported<F>(facade: &F, conservative: bool) -> Option<AnySamplesPassedQuery>
                               where F: Facade
    {
        if conservative {
            if let Some(q) = RawQuery::new_if_supported(facade,
                                                        QueryType::AnySamplesPassedConservative)
            {
                return Some(AnySamplesPassedQuery { query: q });
            }
        }

        if let Some(q) = RawQuery::new_if_supported(facade, QueryType::AnySamplesPassed) {
            return Some(AnySamplesPassedQuery { query: q });
        } else if let Some(q) = RawQuery::new_if_supported(facade, QueryType::SamplesPassed) {
            return Some(AnySamplesPassedQuery { query: q });
        } else {
            return None;
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
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    pub fn new_if_supported<F>(facade: &F) -> Option<PrimitivesGeneratedQuery> where F: Facade {
        RawQuery::new_if_supported(facade, QueryType::PrimitivesGenerated)
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
    /// Builds a new query. Returns `None` if the backend doesn't support this type.
    pub fn new_if_supported<F>(facade: &F) -> Option<TransformFeedbackPrimitivesWrittenQuery>
                               where F: Facade
    {
        RawQuery::new_if_supported(facade, QueryType::TransformFeedbackPrimitivesWritten)
                .map(|q| TransformFeedbackPrimitivesWrittenQuery { query: q })
    }
}

impl_helper!(TransformFeedbackPrimitivesWrittenQuery, u32, get_u32);
