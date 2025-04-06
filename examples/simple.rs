
use concurrent::{self as coc, Queue, TaskCurrier, TaskKind, Which};

fn main() {
    test_c1r1();
}

fn test_c1r1() {
    println!("----- test pool -----");
    let mut pool = coc::Pool::new();
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    pool.add(TaskCurrier::from((print1,Which::default(),TaskKind::Normal)));
    let id = pool.add(TaskCurrier::from((print3,Which::default(),TaskKind::Exit)));
    pool.add(TaskCurrier::from((print2,Which::new(id,0),TaskKind::Normal)));
    pool.insert_thread_from(qid);
    pool.wait();
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
