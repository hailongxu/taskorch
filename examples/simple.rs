
use concurrent::{self as coc, Queue};

fn main() {
    test_task_added_in_current();
    test_task_added_in_another();
}

fn test_task_added_in_current() {
    println!("----- add task in current thread -----");
    let mut queue = Queue::new();
    queue.add(print1);
    queue.add(||print2(&3));
    queue.add(||println!("task #2"));
    queue.add_exit(||println!("exit2"));
    coc::spawn_thread(&queue).wait().unwrap();
}

fn test_task_added_in_another() {
    use std::thread;

    println!("----- add task in another thread -----");
    let mut queue = Queue::new();
    queue.add(print1);
    queue.add(||print2(&4));

    let mut pool = coc::Pool::new();
    coc::spawn_thread(&queue).collect_into(&mut pool);

    // add task in another thread
    thread::spawn({
        let mut queue = queue.clone();
        move || {
        queue.add(||print3(8));
        queue.add_exit(exit);
    }}).join().unwrap();

    pool.wait();
}

use std::thread;
fn print1() {
    println!("[{:?}] task #1 ",
        thread::current().id());
}
fn print2(a:&i32) {
    println!("[{:?}] task #2 {a} ",
        thread::current().id());
}
fn print3(a:i32) {
    println!("[{:?}] task #3 added in another thread {a} ",
        thread::current().id());
}
fn exit() {
    println!("[{:?}] exit task, after this, thread will exit.",
        thread::current().id());
}

