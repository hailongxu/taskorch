
use concurrent::{self as coc, Queue, Which};

fn main() {
    test_pool();
}

fn test_pool() {
    println!("----- test pool -----");
    let mut pool = coc::Pool::new();
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    pool.add(print1,Default::default());
    let id = pool.addc1(print3,Default::default());
    pool.add(print2, Which::new(id, 0));
    // queue.add(print1);
    // queue.add(||print2(&3));
    // queue.add(||println!("task #2"));
    // pool.add_exit(||println!("exit2"));
    pool.insert_thread_from(qid);
    pool.wait();
}

fn test_task_added_in_another() {
    // use std::thread;

    // println!("----- add task in another thread -----");
    // let mut queue = Queue::new();
    // queue.add(print1);
    // queue.add(||print2(&4));

    // let mut pool = coc::Pool::new();
    // coc::spawn_thread(&queue).collect_into(&mut pool);

    // // add task in another thread
    // thread::spawn({
    //     let mut queue = queue.clone();
    //     move || {
    //     queue.add(||print3(8));
    //     queue.add_exit(exit);
    // }}).join().unwrap();

    // pool.wait();
}

use std::thread;
fn print1() {
    println!("[{:?}] task #1 ",
        thread::current().id());
}
fn print2()->i32 {
    let r = 5;
    println!("[{:?}] task #2  r={r:?}",
        thread::current().id());
    r
}
fn print3(a:i32) {
    println!("[{:?}] task cond #3 wait the cond {a} ",
        thread::current().id());
}
fn exit() {
    println!("[{:?}] exit task, after this, thread will exit.",
        thread::current().id());
}
