use std::mem;

use version::Api;
use version::Version;
use context::CommandContext;
use backend::Facade;
use BufferViewExt;
use ContextExt;
use TransformFeedbackSessionExt;
use buffer::{BufferType, BufferView, BufferViewAnySlice};
use index::PrimitiveType;
use program::OutputPrimitives;
use program::Program;
use vertex::Vertex;

use gl;

/// To use transform feedback, you must create a transform feedback session.
///
/// A transform feedback session mutably borrows the buffer where the data will be written.
/// You can only get your data back when the session is destroyed.
#[derive(Debug)]
pub struct TransformFeedbackSession<'a> {
    buffer: BufferViewAnySlice<'a>,
    program: &'a Program,
}

/// Error that can happen when creating a `TransformFeedbackSession`.
#[derive(Debug, Clone)]
pub enum TransformFeedbackSessionCreationError {
    /// Transform feedback is not supported by the OpenGL implementation.
    NotSupported,
    
    /// The format of the output doesn't match what the program is expected to output.
    WrongVertexFormat,
}

/// Returns true if transform feedback is supported by the OpenGL implementation.
pub fn is_transform_feedback_supported<F>(facade: &F) -> bool where F: Facade {
    let context = facade.get_context();

    context.get_version() >= &Version(Api::Gl, 3, 0) ||
    context.get_version() >= &Version(Api::GlEs, 3, 0) ||
    context.get_extensions().gl_ext_transform_feedback
}

impl<'a> TransformFeedbackSession<'a> {
    /// Builds a new transform feedback session.
    ///
    /// TODO: this constructor should ultimately support passing multiple buffers of different
    ///       types
    pub fn new<F, V>(facade: &F, program: &'a Program, buffer: &'a mut BufferView<V>)
                     -> Result<TransformFeedbackSession<'a>, TransformFeedbackSessionCreationError>
                     where F: Facade, V: Vertex + Copy + Send + 'static
    {
        if !is_transform_feedback_supported(facade) {
            return Err(TransformFeedbackSessionCreationError::NotSupported);
        }

        if !program.transform_feedback_matches(&<V as Vertex>::build_bindings(), 
                                               mem::size_of::<V>())
        {
            return Err(TransformFeedbackSessionCreationError::WrongVertexFormat); 
        }

        Ok(TransformFeedbackSession {
            buffer: buffer.as_slice_any(),
            program: program,
        })
    }
}

impl<'a> TransformFeedbackSessionExt for TransformFeedbackSession<'a> {
    fn bind(&self, mut ctxt: &mut CommandContext, draw_primitives: PrimitiveType) {
        // TODO: check that the state matches what is required
        if ctxt.state.transform_feedback_enabled.is_some() {
            unimplemented!();
        }

        self.buffer.indexed_bind_to(ctxt, BufferType::TransformFeedbackBuffer, 0);

        unsafe {
            let primitives = match (self.program.get_output_primitives(), draw_primitives) {
                (Some(OutputPrimitives::Points), _) => gl::POINTS,
                (Some(OutputPrimitives::Lines), _) => gl::LINES,
                (Some(OutputPrimitives::Triangles), _) => gl::TRIANGLES,
                (None, PrimitiveType::Points) => gl::POINTS,
                (None, PrimitiveType::LinesList) => gl::LINES,
                (None, PrimitiveType::LinesListAdjacency) => gl::LINES,
                (None, PrimitiveType::LineStrip) => gl::LINES,
                (None, PrimitiveType::LineStripAdjacency) => gl::LINES,
                (None, PrimitiveType::TrianglesList) => gl::TRIANGLES,
                (None, PrimitiveType::TrianglesListAdjacency) => gl::TRIANGLES,
                (None, PrimitiveType::TriangleStrip) => gl::TRIANGLES,
                (None, PrimitiveType::TriangleStripAdjacency) => gl::TRIANGLES,
                (None, PrimitiveType::TriangleFan) => gl::TRIANGLES,
                (None, PrimitiveType::Patches { .. }) => unreachable!(),
            };

            ctxt.gl.BeginTransformFeedback(primitives);
            ctxt.state.transform_feedback_enabled = Some(primitives);
            ctxt.state.transform_feedback_paused = false;
        }
    }

    fn unbind(mut ctxt: &mut CommandContext) {
        if ctxt.state.transform_feedback_enabled.is_none() {
            return;
        }

        unsafe {
            ctxt.gl.EndTransformFeedback();
            ctxt.state.transform_feedback_enabled = None;
            ctxt.state.transform_feedback_paused = false;
        }
    }

    fn ensure_buffer_out_of_transform_feedback(mut ctxt: &mut CommandContext, buffer: gl::types::GLuint) {
        if ctxt.state.transform_feedback_enabled.is_none() {
            return;
        }

        let mut needs_unbind = false;
        for elem in ctxt.state.indexed_transform_feedback_buffer_bindings.iter_mut() {
            if elem.buffer == buffer {
                needs_unbind = true;
                break;
            }
        }

        if needs_unbind {
            TransformFeedbackSession::unbind(ctxt);
        }
    }
}

impl<'a> Drop for TransformFeedbackSession<'a> {
    fn drop(&mut self) {
        // FIXME: since the session can be mem::forget'ed, the code in buffer/alloc.rs should make
        //        sure that the buffer isn't in use for transform feedback
    }
}
