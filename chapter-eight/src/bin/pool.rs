use futures::prelude::*;
use futures::task::{Context, Poll, SpawnExt, LocalSpawnExt};
use futures::channel::oneshot;
use futures::executor::{block_on, LocalPool, ThreadPool};

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::pin::Pin;

#[derive(Clone, Copy, Debug)]
enum Status {
    Loading,
    FetchingData,
    Loaded,
}

#[derive(Clone, Copy, Debug)]
struct Container {
    name: &'static str,
    status: Status,
    ticks: u64,
}

impl Container {
    fn new(name: &'static str) -> Self {
        Container {
            name: name,
            status: Status::Loading,
            ticks: 3,
        }
    }

    // simulate ourselves retreiving a score from a remote database
    fn pull_score(&mut self) -> u32 {
        self.status = Status::Loaded;
        thread::sleep(Duration::from_secs(self.ticks));
        100
    }
}

impl Future for Container {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}

const FINISHED: () = ();

fn new_status(unit: &'static str, status: Status) {
    println!("{}: new status: {:?}", unit, status);
}

fn local_until() {
    let mut container = Container::new("acme");

    // setup our green thread pool
    let mut pool = LocalPool::new();

    // create a new future

    // typically, we perform some heavy computational process within this closure
    // such as loading graphic assets, sound, other parts of our framework/library/etc.
    let f = async move {
        container.status = Status::FetchingData;
        container
    };
    println!("container's '{}' current status: {:?}", container.name, container.status);

    container = pool.run_until(f);
    new_status("local_until", container.status);

    // just to demonstrate a simulation of "fetching data over a network"
    println!("Fetching our container's score...");
    let score = block_on(async { container.pull_score() });
    println!("Our container's score is: {:?}", score);

    // see if our status has changed since we fetched our score
    new_status("local_until", container.status);
}

fn local_spawns_completed() {
    let (tx, rx) = oneshot::channel();
    let mut container = Container::new("acme");

    let mut pool = LocalPool::new();
    let spawn = &mut pool.spawner();

    // change our container's status and then send it to our oneshot channel
    spawn.spawn_local(async move {
            container.status = Status::Loaded;
            tx.send(container).unwrap();
            FINISHED
        })
        .unwrap();

    container = pool.run_until(rx).unwrap();
    new_status("local_spanws_completed", container.status);
}

fn local_nested() {
    let mut container = Container::new("acme");

    // we will need Rc (reference counts) since we are referencing multiple owners
    // and we are not using Arc (atomic reference counts) since we are only using
    // a local pool which is on the same thread technically
    let cnt = Rc::new(Cell::new(container));
    let cnt_2 = cnt.clone();

    let mut pool = LocalPool::new();
    let spawn = &mut pool.spawner();
    let spawn_2 = spawn.clone();

    let _ = spawn.spawn_local(async move {
            spawn_2.spawn_local(async move {
                    let mut container = cnt_2.get();
                    container.status = Status::Loaded;

                    cnt_2.set(container);
                    FINISHED
                })
                .unwrap();
            FINISHED
        })
        .unwrap();

    let _ = pool.run();

    container = cnt.get();
    new_status("local_nested", container.status);
}

fn thread_pool() {
    let (tx, rx) = mpsc::sync_channel(2);
    let tx_2 = tx.clone();

    // there are various thread builder options which are referenced at
    // https://docs.rs/futures/latest/futures/executor/struct.ThreadPool.html
    let cpu_pool = ThreadPool::builder()
        .pool_size(2) // default is the number of cpus
        .create()
        .unwrap();

    // We need to box this part since we need the Send +'static trait
    // in order to safely send information across threads
    let _ = cpu_pool.spawn(async move {
        tx.send(1).unwrap();
    })
    .unwrap();

    let f = async move {
        tx_2.send(1).unwrap();
    };
    let _ = cpu_pool.spawn(f).unwrap();

    let cnt = rx.into_iter().count();
    println!("Count should be 2: {:?}", cnt);
}

fn main() {
    println!("local_until():");
    local_until();

    println!("\nlocal_spawns_completed():");
    local_spawns_completed();

    println!("\nlocal_nested():");
    local_nested();

    println!("\nthread_pool():");
    thread_pool();
}
