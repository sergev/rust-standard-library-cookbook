use futures::task::{Context, Poll, Waker};
use futures::lock::BiLock;
use futures::Future;

use std::sync::Arc;
use std::task::Wake;
use std::pin::Pin;

struct FakeWaker;
impl Wake for FakeWaker {
    fn wake(self: Arc<Self>) {}
}

struct Reader<T> {
    lock: BiLock<T>,
}

struct Writer<T> {
    lock: BiLock<T>,
}

fn split() -> (Reader<u32>, Writer<u32>) {
    let (a, b) = BiLock::new(0);
    (Reader { lock: a }, Writer { lock: b })
}

fn main() {
    let waker = Waker::from(Arc::new(FakeWaker));
    let mut cx = Context::from_waker(&waker);

    let (reader, writer) = split();
    println!("Lock should be ready for writer: {}",
             writer.lock.poll_lock(&mut cx).is_ready());
    println!("Lock should be ready for reader: {}",
             reader.lock.poll_lock(&mut cx).is_ready());

    let mut writer_lock = match Pin::new(&mut writer.lock.lock()).poll(&mut cx) {
        Poll::Ready(t) => t,
        _ => panic!("We should be able to lock with writer"),
    };

    println!("Lock should now be pending for reader: {}",
             reader.lock.poll_lock(&mut cx).is_pending());
    *writer_lock = 123;

    let mut lock = reader.lock.lock();
    match Pin::new(&mut lock).poll(&mut cx) {
        Poll::Ready(_) => {
            panic!("The lock should not be lockable since writer has already locked it!")
        }
        _ => println!("Couldn't lock with reader since writer has already initiated the lock"),
    };

    drop(writer_lock);

    let reader_lock = match Pin::new(&mut lock).poll(&mut cx) {
        Poll::Ready(t) => t,
        _ => panic!("We should be able to lock with reader"),
    };

    println!("The new value for the lock is: {}", *reader_lock);

    drop(reader_lock);
    let reunited_value = reader.lock.reunite(writer.lock).unwrap();

    println!("After reuniting our locks, the final value is still: {}",
             reunited_value);
}
