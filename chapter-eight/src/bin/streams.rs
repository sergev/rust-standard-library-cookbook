use std::thread;
use std::pin::Pin;

use futures::prelude::*;
use futures::executor::block_on;
use futures::future::poll_fn;
use futures::stream;
use futures::channel::mpsc;
use futures::task::{Context, Poll};

#[derive(Debug)]
struct QuickStream {
    ticks: usize,
}

impl Stream for QuickStream {
    type Item = usize;

    fn poll_next(mut self: Pin<&mut QuickStream>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.ticks {
            ref mut ticks if *ticks > 0 => {
                *ticks -= 1;
                println!("Ticks left on QuickStream: {}", *ticks);
                Poll::Ready(Some(*ticks))
            }
            _ => {
                println!("QuickStream is closing!");
                Poll::Ready(None)
            }
        }
    }
}

const FINISHED: Poll<()> = Poll::Ready(());

fn quick_streams() {
    let mut quick_stream = QuickStream { ticks: 10 };

    // Collect the first poll() call
    block_on(poll_fn(|cx| {
            let poll = quick_stream.poll_next_unpin(cx);
            if let Poll::Ready(res) = poll {
                println!("Quick stream's value: {:?}", res);
            }
            poll
        }));

    // Collect the second poll() call
    block_on(poll_fn(|cx| {
            let poll = quick_stream.poll_next_unpin(cx);
            if let Poll::Ready(res) = poll {
                println!("Quick stream's value: {:?}", res);
            }
            poll
        }));

    // And now we should be starting from 7 when collecting the rest of the stream
    let result: Vec<_> = block_on(quick_stream.collect::<Vec<_>>());
    println!("quick_streams final result: {:?}", result);
}

fn iterate_streams() {
    use std::borrow::BorrowMut;

    let stream_response = vec![Ok(5), Ok(7), Err(false), Ok(3)];
    let stream_response2 = vec![Ok(5), Ok(7), Err(false), Ok(3)];

    // Useful for converting any of the `Iterator` traits into a `Stream` trait.
    let ok_stream = stream::iter(vec![1, 5, 23, 12]);
    let ok_stream2 = stream::iter(vec![7, 2, 14, 19]);

    let mut result_stream = stream::iter(stream_response);
    let mut result_stream2 = stream::iter(stream_response2);

    let ok_stream_response: Vec<_> = block_on(ok_stream.collect::<Vec<_>>());
    println!("ok_stream_response: {:?}", ok_stream_response);

    let mut count = 1;
    loop {
        match block_on(result_stream.borrow_mut().next()) {
            Some(res) => {
                match res {
                    Ok(r) => println!("iter_result_stream result #{}: {}", count, r),
                    Err(err) => println!("iter_result_stream had an error #{}: {:?}", count, err),
                }
            },
            None => { break }
        }
        count += 1;
    }

    // Alternative way of iterating through an ok stream
    let ok_res: Vec<_> = block_on(ok_stream2.collect::<Vec<_>>());
    for ok_val in ok_res.into_iter() {
        println!("ok_stream2 value: {}", ok_val);
    }

    let _ = block_on(result_stream2.next()).unwrap();
    let _ = block_on(result_stream2.next()).unwrap();
    let err = block_on(result_stream2.next()).unwrap();

    println!("The error for our result_stream2 was: {:?}", err);

    println!("All done.");
}

fn channel_threads() {
    const MAX: usize = 10;
    let (mut tx, rx) = mpsc::channel(0);

    let t = thread::spawn(move || {
        for i in 0..MAX {
            loop {
                if tx.try_send(i).is_ok() {
                    break;
                } else {
                    println!("Thread transaction #{} is still pending!", i);
                }
            }
        }
    });

    let result: Vec<_> = block_on(rx.collect::<Vec<_>>());
    for (index, res) in result.into_iter().enumerate() {
        println!("Channel #{} result: {}", index, res);
    }

    t.join().unwrap();
}

fn channel_error() {
    let (mut tx, mut rx) = mpsc::channel(0);

    tx.try_send("hola").unwrap();

    // This should fail
    match tx.try_send("fail") {
        Ok(_) => println!("This should not have been successful"),
        Err(err) => println!("Send failed! {:?}", err),
    }

    let result = block_on(rx.next()).unwrap();
    println!("The result of the channel transaction is: {}", result);

    // Now we should be able send to the transaction since we poll'ed a result already
    tx.try_send("hasta la vista").unwrap();
    drop(tx);

    let result = block_on(rx.next()).unwrap();
    println!("The next result of the channel transaction is: {}", result);

    // Pulling more should result in None
    let result = block_on(rx.next());
    println!("The last result of the channel transaction is: {:?}", result);
}

fn channel_buffer() {
    let (mut tx, mut rx) = mpsc::channel::<i32>(0);

    let f = poll_fn(move |cx| {
        if !tx.poll_ready(cx).is_ready() {
            panic!("transactions should be ready right away!");
        }

        tx.start_send(20).unwrap();
        if tx.poll_ready(cx).is_pending() {
            println!("transaction is pending...");
        }

        // When we're still in "Pending mode" we should not be able
        // to send more messages/values to the receiver
        if tx.start_send(10).unwrap_err().is_full() {
            println!("transaction could not have been sent to the receiver due \
                      to being full...");
        }

        let result = rx.poll_next_unpin(cx);
        println!("the first result is: {:?}", result);
        println!("is transaction ready? {:?}",
                 tx.poll_ready(cx).is_ready());

        // We should now be able to send another message since we've pulled
        // the first message into a result/value/variable.
        if !tx.poll_ready(cx).is_ready() {
            panic!("transaction should be ready!");
        }

        tx.start_send(22).unwrap();
        let result = rx.poll_next_unpin(cx);
        println!("new result for transaction is: {:?}", result);

        FINISHED
    });

    block_on(f);
}

fn channel_threads_blocking() {
    let (mut tx, mut rx) = mpsc::channel::<i32>(0);
    let (tx_2, mut rx_2) = mpsc::channel::<()>(2);

    let t = thread::spawn(move || {
        let mut tx_2 = tx_2.sink_map_err(|_| panic!());

        let (_r1, _r2) = block_on(future::join(tx.send(10), tx_2.send(())));
        let (_r1, _r2) = block_on(future::join(tx.send(30), tx_2.send(())));
    });

    block_on(rx_2.next()).unwrap();
    let result = block_on(rx.next()).unwrap();
    println!("The first number that we sent was: {}", result);

    drop(block_on(rx_2.next()));
    let result = block_on(rx.next());
    println!("The second number that we sent was: {:?}", result);

    t.join().unwrap();
}

fn channel_unbounded() {
    const MAX_SENDS: u32 = 5;
    const MAX_THREADS: u32 = 4;
    let (tx, rx) = mpsc::unbounded::<i32>();

    let t = thread::spawn(move || {
        let result: Vec<_> = block_on(rx.collect::<Vec<_>>());
        for item in result.iter() {
            println!("channel_unbounded: results on rx: {:?}", item);
        }
    });

    for _ in 0..MAX_THREADS {
        let tx = tx.clone();

        thread::spawn(move || {
            for _ in 0..MAX_SENDS {
                tx.unbounded_send(1).unwrap();
            }
        });
    }

    drop(tx);

    t.join().unwrap();
}

fn main() {
    println!("quick_streams():");
    quick_streams();

    println!("\niterate_streams():");
    iterate_streams();

    println!("\nchannel_threads():");
    channel_threads();

    println!("\nchannel_error():");
    channel_error();

    println!("\nchannel_buffer():");
    channel_buffer();

    println!("\nchannel_threads_blocking():");
    channel_threads_blocking();

    println!("\nchannel_unbounded():");
    channel_unbounded();
}
