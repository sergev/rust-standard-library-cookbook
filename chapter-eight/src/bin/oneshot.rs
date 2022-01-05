use futures::channel::oneshot;
use futures::executor::{block_on, block_on_stream};
use futures::future::poll_fn;
use futures::stream::FuturesOrdered;
use futures::task::Poll;

const FINISHED: Poll<()> = Poll::Ready(());

fn send_example() {
    // First, we'll need to initiate some oneshot channels like so:
    let (tx_1, rx_1) = oneshot::channel::<u32>();
    let (tx_2, rx_2) = oneshot::channel::<u32>();
    let (tx_3, rx_3) = oneshot::channel::<u32>();

    // We can decide if we want to sort our futures by FIFO (futures_ordered)
    // or if the order doesn't matter (futures_unordered)
    // Note: All futured_ordered()'ed futures must be set as a Box type
    let mut ordered_stream = vec![
        rx_1,
        rx_2,
    ].into_iter().collect::<FuturesOrdered<_>>();

    ordered_stream.push(rx_3);

    // unordered example:
    // let unordered_stream = vec![rx_1, rx_2, rx_3].into_iter().collect::<FuturesUnordered<_>>();

    // Call an API, database, etc. and return the values (in our case we're typecasting to u32)
    tx_1.send(7).unwrap();
    tx_2.send(12).unwrap();
    tx_3.send(3).unwrap();

    let ordered_results: Vec<_> = block_on_stream(ordered_stream).collect();
    println!("Ordered stream results: {:?}", ordered_results);
}

fn check_if_closed() {
    let (tx, rx) = oneshot::channel::<u32>();

    println!("Is our channel canceled? {:?}", tx.is_canceled());
    drop(rx);

    println!("Is our channel canceled now? {:?}", tx.is_canceled());
}

fn check_if_ready() {
    let (mut tx, rx) = oneshot::channel::<u32>();
    let mut rx = Some(rx);

    block_on(poll_fn(|cx| {
            println!("Is the transaction pending? {:?}",
                     tx.poll_canceled(cx).is_pending());
            drop(rx.take());

            let is_ready = tx.poll_canceled(cx).is_ready();
            let is_pending = tx.poll_canceled(cx).is_pending();

            println!("Are we ready? {:?} This means that the pending should be false: {:?}",
                     is_ready,
                     is_pending);
            FINISHED
        }));
}

fn main() {
    println!("send_example():");
    send_example();

    println!("\ncheck_if_closed():");
    check_if_closed();

    println!("\ncheck_if_ready():");
    check_if_ready();
}
