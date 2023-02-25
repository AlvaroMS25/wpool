use std::thread::{sleep};
use std::time::Duration;
use crate::builder::WorkerPoolBuilder;
use crate::driver::Driver;
use crate::handle::Handle;
use super::*;

#[test]
fn hello_world() {
    WorkerPoolBuilder::new()
        .build_owned().unwrap();

    spawn_detached(|| {
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

#[test]
fn use_context_guard() {
    let handle = WorkerPoolBuilder::new().build_owned().unwrap();
    std::thread::spawn(move || {
        let _guard = handle.enter_context();
        crate::spawn_detached(|| unreachable!());
    }).join().unwrap();
}

#[test]
fn forget_guard() {
    let handle = WorkerPoolBuilder::new().build_owned().unwrap();
    std::thread::spawn(move || {
        std::mem::forget(handle.enter_context());
        crate::spawn_detached(|| unreachable!());
    }).join().unwrap();
}

#[test]
#[should_panic]
fn drop_context_guard() {
    let handle = WorkerPoolBuilder::new().build_owned().unwrap();
    std::thread::spawn(move || {
        let _ = handle.enter_context();
        crate::spawn_detached(|| {});
    }).join().unwrap();
}

#[test]
fn scope() {
    let handle = WorkerPoolBuilder::new().threads(4).build().unwrap();

    handle.scoped(|scope| {
        scope.spawn(|| {
            println!("Sleeping 3 seconds");
            sleep(Duration::from_secs(3));
            println!("Finished");
        });

        scope.spawn(|| {
            println!("Sleeping 6 seconds");
            sleep(Duration::from_secs(6));
            println!("Finished");
        });

        scope.spawn(|| {
            println!("Sleeping 9 seconds");
            sleep(Duration::from_secs(9));
            println!("Finished");
        });

        scope.spawn(|| {
            println!("Sleeping 15 seconds");
            sleep(Duration::from_secs(15));
            println!("Finished");
        });

        println!("Ended scope spawns");
    });

    println!("Scope exited");
}

#[test]
fn scoped_wait() {
    let handle = WorkerPoolBuilder::new().build_owned().unwrap();
    _scoped(handle);
}

fn _scoped(handle: Handle) {
    let mut num = 0;
    let mut string = String::new();

    let res: crate::error::Result<()> = handle.scoped(|scope| {
        scope.spawn(|| {
            num+=165;
        });

        scope.spawn(|| {
            string = format!("Hello world");
        });

        let _ = scope.spawn(|| {
            2
        }).join()?;

        let _ = scope.spawn(|| {
            sleep(Duration::from_secs(2));
            32
        }).join()?;

        Ok(())
    });

    println!("Values -> {:?}\nReturn -> {:?}", (num, string), res);
}

#[test]
fn multiple_scopes() {
    let handle = WorkerPoolBuilder::new()
        .threads(5000)
        .before_task(|| println!("Starting task"))
        .build().unwrap();
    let mut handles = Vec::new();
    for _ in 0..1000 {
        handles.push(handle.spawn(move || {
            println!("Starting scope");
            let handle = Handle::current();
            _scoped(handle.clone());
            println!("Scope ended");
        }));
    }

    handles.into_iter().map(JoinHandle::wait).collect::<Result<Vec<()>, _>>().unwrap();
}

#[test]
fn parallel_multiple() {
    let _ = (0..5000).map(|_| std::thread::spawn(multiple_scopes))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join())
        .collect::<Vec<_>>();
}
