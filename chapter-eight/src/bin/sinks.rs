use futures::prelude::*;
use futures::future::poll_fn;
use futures::executor::block_on;
use futures::sink::SinkExt;
use futures::stream::iter;
use futures::task::{Waker, Context, Poll};

use std::mem;
use std::pin::Pin;
use std::convert::Infallible;

fn vector_sinks() {
    let mut vector = Vec::new();
    let result = Pin::new(&mut vector).start_send(0).unwrap();
    let result2 = Pin::new(&mut vector).start_send(7).unwrap();

    println!("vector_sink: results of sending should both be Ok(()): {:?} and {:?}",
             result,
             result2);
    println!("The entire vector is now {:?}", vector);

    // Now we need to flush our vector sink.
    let flush = block_on(SinkExt::flush(&mut vector));
    println!("Our flush value: {:?}", flush);
    println!("Our vector value: {:?}", flush.unwrap());

    let mut vector = Vec::new();
    {
        let mut pinned_vector = Pin::new(&mut vector);
        let _result = pinned_vector.send(2);
        // safe to unwrap since we know that we have not flushed the sink yet
        let result = pinned_vector.send(4);

        println!("Result of send(): {:?}", result);

        // TODO: Cannot print vector here, as it's being borrowed.
        // TODO: println!("Our vector after send(): {:?}", vector);

        block_on(result).unwrap();
    }
    println!("Our vector should already have one element: {:?}", vector);

    let _result = block_on(Pin::new(&mut vector).send(2)).unwrap();
    println!("We can still send to our stick to ammend values: {:?}",
             vector);

    let mut vector = Vec::new();
    let mut stream = stream::iter(vec![1, 2, 3]).map(Ok);
    let send_all = vector.send_all(&mut stream);
    println!("The value of vector's send_all: {:?}", send_all);

    // Add some more elements to our vector...
    block_on(send_all).unwrap();
    let result = block_on(vector.send_all(&mut stream::iter(vec![0, 6, 7]).map(Ok))).unwrap();
    println!("send_all's return value: {:?}", result);
}

fn mapping_sinks() {
    let mut sink = Vec::new().with(|elem: i32| future::ok::<i32, Infallible>(elem * elem));

    block_on(sink.send(0)).unwrap();
    block_on(sink.send(3)).unwrap();
    block_on(sink.send(5)).unwrap();
    println!("sink with() value: {:?}", sink.into_inner());

    let mut sink = Vec::new().with_flat_map(|elem| stream::iter(vec![elem; elem]).map(Ok));

    block_on(sink.send(0)).unwrap();
    block_on(sink.send(3)).unwrap();
    block_on(sink.send(5)).unwrap();
    block_on(sink.send(7)).unwrap();
    println!("sink with_flat_map() value: {:?}", sink.into_inner());
}

fn fanout() {
    let sink1 = vec![];
    let sink2 = vec![];
    let mut sink = sink1.fanout(sink2);
    let mut stream = iter(vec![1, 2, 3]).map(Ok);
    block_on(sink.send_all(&mut stream)).unwrap();
    let (sink1, sink2) = sink.into_inner();

    println!("sink1 values: {:?}", sink1);
    println!("sink2 values: {:?}", sink2);
}

#[derive(Debug)]
struct ManualSink<T> {
    data: Vec<T>,
    waiting_tasks: Vec<Waker>,
}

impl<T: Unpin> Sink<T> for ManualSink<T> {
    type Error = ();

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.data.push(item);
        Ok(())
    }

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.data.is_empty() {
            Poll::Ready(Ok(()))
        } else {
            self.waiting_tasks.push(cx.waker().clone());
            Poll::Pending
        }
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<T> ManualSink<T> {
    fn new() -> ManualSink<T> {
        ManualSink {
            data: Vec::new(),
            waiting_tasks: Vec::new(),
        }
    }

    fn force_flush(&mut self) -> Vec<T> {
        for task in self.waiting_tasks.clone() {
            println!("Executing a task before replacing our values");
            task.wake();
        }

        mem::replace(&mut self.data, vec![])
    }
}

fn manual_flush() {
    let mut sink = ManualSink::new().with(|x| future::ok::<i32, ()>(x));
    sink.start_send_unpin(3).unwrap();

    let f = poll_fn(move |cx| {
        // Try to flush our ManualSink
        let _ = sink.poll_flush_unpin(cx);
        println!("Our sink after trying to flush: {:?}", sink.get_ref());

        sink.start_send_unpin(7).unwrap();
        let _ = sink.poll_flush_unpin(cx);

        let results = sink.get_mut().force_flush();
        println!("Sink data after manually flushing: {:?}",
                 sink.get_ref().data);
        println!("Final results of sink: {:?}", results);

        Poll::Ready(Some(()))
    });

    block_on(f).unwrap();
}

fn main() {
    println!("vector_sinks():");
    vector_sinks();

    println!("\nmapping_sinks():");
    mapping_sinks();

    println!("\nfanout():");
    fanout();

    println!("\nmanual_flush():");
    manual_flush();
}
