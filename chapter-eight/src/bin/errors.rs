use futures::prelude::*;
use futures::executor::block_on;
use futures::stream;
use futures::task::{Context, Poll};
use futures::future::{err, Ready};

use std::pin::Pin;

struct MyFuture {}
impl MyFuture {
    fn new() -> Self {
        MyFuture {}
    }
}

fn map_error_example() -> Ready<Result<(), &'static str>> {
    err::<(), &'static str>("map_error has occurred")
}

fn err_into_example() -> Ready<Result<(), u8>> {
    err::<(), u8>(1)
}

fn or_else_example() -> Ready<Result<(), &'static str>> {
    err::<(), &'static str>("or_else error has occurred")
}

impl Future for MyFuture {
    type Output = Result<(), Box<dyn std::error::Error>>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(Err("A generic error goes here".into()))
    }
}

struct FuturePanic {}

impl Future for FuturePanic {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        panic!("It seems like there was a major issue with catch_unwind_example")
    }
}

fn using_recover() {
    let _f = MyFuture::new();
/*
    // TODO: recover() method is not available for Future anymore
    let f_recover = f.recover::<_>(|err| {
        println!("An error has occurred: {}", err);
        ()
    });

    block_on(f_recover).unwrap();
*/
}

fn map_error() {
    let map_fn = |err| format!("map_error_example: {}", err);

    if let Err(e) = block_on(map_error_example().map_err(map_fn)) {
        println!("block_on error: {}", e)
    }
}

fn err_into() {
    if let Err(e) = block_on(err_into_example().err_into::<u32>()) {
        println!("block_on error code: {:?}", e)
    }
}

fn or_else() {
    if let Err(e) = block_on(or_else_example()
        .or_else(|_| { err("changed or_else's error message") })) {
        println!("block_on error: {}", e)
    }
}

fn catch_unwind() {
    let f = FuturePanic {};

    if let Err(e) = block_on(f.catch_unwind()) {
        let err = e.downcast::<&'static str>().unwrap();
        println!("block_on error: {:?}", err)
    }
}

fn stream_panics() {
    let stream_ok = stream::iter(vec![Some(1), Some(7), None, Some(20)]);
    // We panic on "None" values in order to simulate a stream that panics
    let stream_map = stream_ok.map(|o| o.unwrap());

    // We can use catch_unwind() for catching panics
    let stream = stream_map.catch_unwind();
    let stream_results: Vec<_> = block_on(stream.collect::<Vec<_>>());

    // Here we can use the partition() function to separate the Ok and Err values
    let (oks, errs): (Vec<_>, Vec<_>) = stream_results.into_iter().partition(Result::is_ok);
    let ok_values: Vec<_> = oks.into_iter().map(Result::unwrap).collect();
    let err_values: Vec<_> = errs.into_iter().map(Result::unwrap_err).collect();

    println!("Panic's Ok values: {:?}", ok_values);
    println!("Panic's Err values: {:?}", err_values);
}

fn main() {
    println!("using_recover():");
    using_recover();

    println!("\nmap_error():");
    map_error();

    println!("\nerr_into():");
    err_into();

    println!("\nor_else():");
    or_else();

    println!("\ncatch_unwind():");
    catch_unwind();

    println!("\nstream_panics():");
    stream_panics();
}
