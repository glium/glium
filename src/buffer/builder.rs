use backend::{Context, Facade};
use buffer::sub::{self, SubBuffer};
use buffer::alloc::Buffer;
use buffer::BufferType;

use std::mem;
use std::rc::Rc;

/// Struct that allows you to build multiple subbuffers stored inside the same buffer.
///
/// # Example
///
/// ```no_run
/// # #[macro_use]
/// # extern crate glium;
/// # fn main() {
/// # let display: glium::Display = unsafe { std::mem::uninitialized() };
/// # #[derive(Copy, Clone)] struct Vertex { field: f32 } implement_vertex!(Vertex, field);
/// let (vb1, vb2) = glium::buffer::Builder::new(&display)
///                         .add_empty(12)    // vb1
///                         .add_empty(16)    // vb2
///                         .build();
///
/// // rustc needs some hints about the types of the buffers,
/// // so you may have to write something like this:
/// let vb1: glium::vertex::VertexBuffer<Vertex> = vb1;
/// let vb2: glium::vertex::VertexBuffer<Vertex> = vb2;
/// # }
/// ```
///
pub struct Builder<R> {
    context: Rc<Context>,
    data: R,
}

/// An input parameter stored inside the `Builder` and that will be processed when building
/// the buffers.
pub enum BuilderParam<'a, T: 'a> {
    /// Some initial data to be put in the buffer.
    Data(&'a [T]),
    /// Empty space.
    Empty(usize),
}

impl<'a, T: 'a> BuilderParam<'a, T> {
    /// Returns the number of elements requested.
    fn get_num_elements(&self) -> usize {
        match self {
            &BuilderParam::Data(ref data) => data.len(),
            &BuilderParam::Empty(num) => num,
        }
    }

    /// Returns the size in bytes of this part of the buffer.
    fn get_buffer_size(&self) -> usize {
        match self {
            &BuilderParam::Data(ref data) => data.len() * mem::size_of::<T>(),
            &BuilderParam::Empty(num) => num * mem::size_of::<T>(),
        }
    }
}

impl Builder<()> {
    /// Builds a new empty builder.
    pub fn new<F>(facade: &F) -> Builder<()> where F: Facade {
        Builder {
            context: facade.get_context().clone(),
            data: (),
        }
    }
}

impl<R> Builder<R> {
    /// Adds an empty buffer to the list of buffers to create.
    pub fn add_empty<'a, O>(self, len: usize) -> Builder<<R as BuilderTupleAdd<'a, O>>::Output>
                        where R: BuilderTupleAdd<'a, O>
    {
        Builder {
            context: self.context,
            data: self.data.add_empty(len),
        }
    }

    /// Adds a buffer with some data to the list of buffers to create.
    pub fn add_data<'a, O>(self, data: &'a [O]) -> Builder<<R as BuilderTupleAdd<'a, O>>::Output>
                           where R: BuilderTupleAdd<'a, O>
    {
        Builder {
            context: self.context,
            data: self.data.add_data(data),
        }
    }

    /// Consumes the builder and returns the generated buffers.
    pub fn build<O>(self) -> O where R: BuilderParamsList<O> {
        self.data.build(&self.context)
    }
}

/// List of `BuilderParam`. The template parameter `O` represents the possible buffers list output
/// when `build` is called.
pub trait BuilderParamsList<O> {
    /// Builds the list of buffers.
    fn build(self, context: &Rc<Context>) -> O;
}

/// List of `BuilderParam` that can add a buffer whose content is of type `T` to their list.
pub trait BuilderTupleAdd<'a, T> {
    /// The resulting type after a new buffer is added to the list.
    type Output;

    /// Adds an empty buffer to the list and returns the new list.
    fn add_empty(self, usize) -> Self::Output;
    /// Adds a buffer with data to the list and returns the new list.
    fn add_data(self, &'a [T]) -> Self::Output;
}

macro_rules! implement {
    () => {};

    ($first_in:ident $first_out:ident) => {
        implement!($first_in $first_out,);
    };

    ($first_in:ident $first_out:ident, $($rest_in:ident $rest_out:ident),*) => {
        impl<'a, $first_in: 'a, $($rest_in: 'a,)* $first_out $(, $rest_out)*>
            BuilderParamsList<($first_out $(, $rest_out)*)>
            for (BuilderParam<'a, $first_in>, $(BuilderParam<'a, $rest_in>),*)
            where $first_out: From<SubBuffer<$first_in>> $(, $rest_out: From<SubBuffer<$rest_in>>)*,
                  $first_in: Copy + Send + 'static $(, $rest_in: Copy + Send + 'static)*
        {
            #[allow(non_snake_case)]
            #[allow(unused_mut)]
            fn build(self, context: &Rc<Context>) -> ($first_out $(, $rest_out)*) {
                let ($first_in, $($rest_in),*) = self;
                let size = $first_in.get_buffer_size() $(+ $rest_in.get_buffer_size())*;

                // TODO: buffer type?
                let buffer = Buffer::empty(context, BufferType::ArrayBuffer, size, false).unwrap();
                let buffer = Rc::new(buffer);

                let mut offset = 0;

                $(
                    let $rest_out = sub::build_sub_buffer(buffer.clone(), offset,
                                                          $rest_in.get_num_elements());
                    offset += $rest_in.get_buffer_size();
                    match $rest_in {
                        BuilderParam::Empty(_) => (),
                        BuilderParam::Data(data) => $rest_out.write(data),
                    };
                    let $rest_out = $rest_out.into();
                )*

                let $first_out = sub::build_sub_buffer(buffer, offset,
                                                       $first_in.get_num_elements());
                match $first_in {
                    BuilderParam::Empty(_) => (),
                    BuilderParam::Data(data) => $first_out.write(data),
                };
                let $first_out = $first_out.into();

                ($first_out $(, $rest_out)*)
            }
        }

        impl<'a, T: 'a, $first_in: 'a $(, $rest_in: 'a)*> BuilderTupleAdd<'a, T>
            for (BuilderParam<'a, $first_in>, $(BuilderParam<'a, $rest_in>),*)
        {
            type Output = (BuilderParam<'a, $first_in>, $(BuilderParam<'a, $rest_in>,)* BuilderParam<'a, T>);

            #[allow(non_snake_case)]
            fn add_empty(self, len: usize) -> Self::Output {
                let ($first_in, $($rest_in),*) = self;
                ($first_in, $($rest_in,)* BuilderParam::Empty(len))
            }

            #[allow(non_snake_case)]
            fn add_data(self, data: &'a [T]) -> Self::Output {
                let ($first_in, $($rest_in),*) = self;
                ($first_in, $($rest_in,)* BuilderParam::Data(data))
            }
        }

        implement!($($rest_in $rest_out),*);
    };
}

implement!(I1 O1, I2 O2, I3 O3, I4 O4, I5 O5, I6 O6, I7 O7, I8 O8, I9 O9);

impl<'a, T: 'a> BuilderTupleAdd<'a, T> for () {
    type Output = (BuilderParam<'a, T>,);

    fn add_empty(self, len: usize) -> Self::Output {
        (BuilderParam::Empty(len),)
    }

    fn add_data(self, data: &'a [T]) -> Self::Output {
        (BuilderParam::Data(data),)
    }
}
