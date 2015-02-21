use std::collections::VecDeque;
use std::sync::{atomic, mpsc, Arc, Condvar, Mutex};
use std::{mem, ptr};
use context::CommandContext;

use GliumCreationError;
use glutin;

const CLOSURES_MAX_SIZE: usize = 64;
const MAX_QUEUE_SIZE: usize = 64;

pub struct Sender {
    queue: Arc<Queue>,
}

pub struct Receiver {
    queue: Arc<Queue>,
    closed: atomic::AtomicBool,
}

struct Queue {
    queue: Mutex<VecDeque<InternalMessage>>,
    condvar: Condvar,
}

pub enum Message {
    EndFrame,
    Stop,
    Execute(Exec),
    Rebuild(glutin::WindowBuilder<'static>, mpsc::Sender<Result<(), GliumCreationError>>),
}

pub struct Exec {
    /// storage for the closure
    data: [usize; CLOSURES_MAX_SIZE],
    /// function used to execute the closure
    call_fn: fn([usize; CLOSURES_MAX_SIZE], CommandContext),
}

enum InternalMessage {
    EndFrame,
    Stop,
    Execute {
        /// storage for the closure
        data: [usize; CLOSURES_MAX_SIZE],
        /// function used to execute the closure
        call_fn: fn([usize; CLOSURES_MAX_SIZE], CommandContext),
    },
    Rebuild(glutin::WindowBuilder<'static>, mpsc::Sender<Result<(), GliumCreationError>>),
}

pub fn channel() -> (Sender, Receiver) {
    let queue_sender = Arc::new(Queue {
        queue: Mutex::new(VecDeque::with_capacity(MAX_QUEUE_SIZE)),
        condvar: Condvar::new(),
    });

    let queue_receiver = queue_sender.clone();

    (Sender {
        queue: queue_sender
    },
    Receiver {
        queue: queue_receiver,
        closed: atomic::AtomicBool::new(false)
    })
}

impl Sender {
    pub fn push<F>(&self, f: F) where F: FnOnce(CommandContext) {       // TODO: + Send + 'static
        assert!(mem::size_of::<F>() <= CLOSURES_MAX_SIZE * mem::size_of::<usize>());

        fn call_fn<F>(data: [usize; CLOSURES_MAX_SIZE], cmd: CommandContext)
                      where F: FnOnce(CommandContext)
        {
            let closure: F = unsafe { ptr::read(data.as_slice().as_ptr() as *const F) };
            closure(cmd);
        }

        let mut data: [usize; CLOSURES_MAX_SIZE] = unsafe { mem::uninitialized() };
        unsafe {
            ptr::copy_nonoverlapping_memory(data.as_mut_slice().as_mut_ptr() as *mut F, &f, 1);
        }

        let message = InternalMessage::Execute {
            data: data,
            call_fn: call_fn::<F>,
        };

        {
            let mut lock = self.queue.queue.lock().unwrap();
            loop {
                if lock.len() >= MAX_QUEUE_SIZE {
                    lock = self.queue.condvar.wait(lock).unwrap();
                    continue;
                }

                lock.push_back(message);
                self.queue.condvar.notify_one();
                break;
            }
        }

        unsafe { mem::forget(f) }
    }

    pub fn push_endframe(&self) {
        let mut lock = self.queue.queue.lock().unwrap();
        lock.push_back(InternalMessage::EndFrame);
        self.queue.condvar.notify_one();
    }

    pub fn push_rebuild(&self, b: glutin::WindowBuilder<'static>,
                        n: mpsc::Sender<Result<(), GliumCreationError>>)
    {
        let mut lock = self.queue.queue.lock().unwrap();
        lock.push_back(InternalMessage::Rebuild(b, n));
        self.queue.condvar.notify_one();
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        let mut lock = self.queue.queue.lock().unwrap();
        lock.push_back(InternalMessage::Stop);
        self.queue.condvar.notify_one();
    }
}

impl Receiver {
    pub fn pop(&self) -> Message {
        let mut lock = self.queue.queue.lock().unwrap();

        loop {
            let msg = lock.pop_front();
            self.queue.condvar.notify_one();

            match msg {
                Some(InternalMessage::EndFrame) => return Message::EndFrame,
                Some(InternalMessage::Execute { data, call_fn }) => return Message::Execute(Exec {
                    data: data, call_fn: call_fn
                }),
                Some(InternalMessage::Stop) => {
                    self.closed.store(true, atomic::Ordering::Release);
                    return Message::Stop;
                },
                Some(InternalMessage::Rebuild(a, b)) => {
                    return Message::Rebuild(a, b);
                },
                None => {
                    if self.closed.load(atomic::Ordering::Acquire) {
                        return Message::Stop;
                    }

                    lock = self.queue.condvar.wait(lock).unwrap();
                }
            }
        }
    }
}

impl Exec {
    pub fn execute(self, ctxt: CommandContext) {
        let f = self.call_fn;
        f(self.data, ctxt);
    }
}

#[cfg(test)]
mod test {
    use super::{Message, channel};

    #[test]
    fn simple() {
        let (sender, receiver) = channel();
        let (tx, rx) = ::std::sync::mpsc::channel();

        sender.push(move |c| {
            tx.send(5).unwrap();
            unsafe { ::std::mem::forget(c) };
        });

        match receiver.pop() {
            Message::Execute(f) => f.execute(unsafe { ::std::mem::uninitialized() }),
            _ => unreachable!()
        };

        assert_eq!(rx.try_recv().unwrap(), 5);
    }

    #[test]
    fn stop_message() {
        let (_, receiver) = channel();

        for _ in (0 .. 5) {
            match receiver.pop() {
                Message::Stop => (),
                _ => unreachable!()
            };
        }
    }
}
