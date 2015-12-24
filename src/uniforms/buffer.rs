use buffer::{Content, BufferType, BufferCreationError};
use buffer::{DynamicBuffer, DynamicBufferSlice, DynamicBufferMutSlice};
use buffer::{ImmutableBuffer, ImmutableBufferSlice, ImmutableBufferMutSlice};
use buffer::{PersistentBuffer, PersistentBufferSlice, PersistentBufferMutSlice};
use uniforms::{AsUniformValue, UniformBlock, UniformValue, LayoutMismatchError};
use program;

use std::ops::{Deref, DerefMut};

use backend::Facade;

pub type UniformBuffer<T> = DynamicBuffer<T>;
pub type UniformBufferSlice<'a, T> = DynamicBufferSlice<'a, T>;
pub type UniformBufferMutSlice<'a, T> = DynamicBufferMutSlice<'a, T>;
pub type ImmutableUniformBuffer<T> = ImmutableBuffer<T>;
pub type ImmutableUniformBufferSlice<'a, T> = ImmutableBufferSlice<'a, T>;
pub type ImmutableUniformBufferMutSlice<'a, T> = ImmutableBufferMutSlice<'a, T>;
pub type PersistentUniformBuffer<T> = PersistentBuffer<T>;
pub type PersistentUniformBufferSlice<'a, T> = PersistentBufferSlice<'a, T>;
pub type PersistentUniformBufferMutSlice<'a, T> = PersistentBufferMutSlice<'a, T>;
