use sync::SyncFence;

pub struct GlobalFence<T> {
    inner: T,
    fence: Option<SyncFence>,
}

impl<T> GlobalFence<T> {
    #[inline]
    pub fn new(inner: T) -> GlobalFence<F> {
        GlobalFence {
            inner: inner,
            fence: None,
        }
    }
}
