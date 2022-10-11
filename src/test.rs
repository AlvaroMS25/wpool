use crate::builder::WorkerPoolBuilder;
use super::*;

#[test]
fn hello_world() {
    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    spawn(|| {
        println!("Hello world");
    });
}

#[test]
fn wait() {
    use std::{thread, time::Duration};

    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    let handle = spawn(|| {
        thread::sleep(Duration::from_millis(500));
        1
    });

    println!("{}", handle.wait().unwrap());
}

#[test]
fn detached() {
    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    spawn_detached(|| {
        println!("Detached detached task");
    });
}

#[test]
fn spawn_inside() {
    use std::{thread, time::Duration};

    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    spawn_detached(|| spawn_detached(|| {
        println!("{}", 2+2);
    }));
}

#[tokio::test]
async fn wait_async() {
    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    let result = spawn(|| 1).await.unwrap();
    println!("{}", result);
}

#[test]
fn periodical() {
    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    spawn_periodic(|| {
        println!("Periodical running");
    }, std::time::Duration::from_secs(3), Some(3));

    std::thread::sleep(std::time::Duration::from_secs(10));
}

#[test]
fn combine() {
    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    spawn_periodic(|| {
        println!("Periodic");
    }, std::time::Duration::from_secs(2), Some(2));

    std::thread::sleep(std::time::Duration::from_millis(3800));
    spawn_detached(|| {
        println!("Detached");
    });

    std::thread::sleep(std::time::Duration::from_secs(4));
}

#[test]
fn shutdown() {
    let handle = WorkerPoolBuilder::new()
        .threads(1).build().unwrap();

    spawn_detached(|| {
        std::thread::sleep(std::time::Duration::from_secs(3));
    });

    let join = spawn(|| {
        unreachable!("Won't run");
    });

    handle.shutdown();

    println!("{:?}", join.wait().unwrap_err());
}

#[test]
#[should_panic]
fn shutdown_spawn() {
    let handle = WorkerPoolBuilder::new()
        .build_owned().unwrap();

    handle.clone().shutdown();
    handle.spawn_detached(|| {
        unreachable!();
    });
}
