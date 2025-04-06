# concurrent-pool
The entire concurrency library can generally be divided into three main components:

- **Queue** — The queue holds tasks that are waiting to be processed. It acts as a buffer where tasks are stored until they can be executed.
- **Threads** — Threads are responsible for executing the tasks retrieved from the queue.
- **Resource Pool** — Responsible for the lifecycle management of both the Queue and the Threads, establishing a mapping between names and instances.

## example
```rust
    use concurrent as coc;
    println!("----- test pool -----");
    let mut pool = coc::Pool::new();
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    pool.add(TaskCurrier::from((print1,Which::default(),TaskKind::Normal)));
    let id = pool.add(TaskCurrier::from((print3,Which::default(),TaskKind::Exit)));
    pool.add(TaskCurrier::from((print2,Which::new(id,0),TaskKind::Normal)));
    pool.insert_thread_from(qid);
    pool.wait();

    fn print1() {
        println!("[{:?}] task #1 ", thread::current().id());
    }
    fn print2()->i32 {
        let r = 5;
        println!("[{:?}] task #2  r={r:?}", thread::current().id());
        r
    }
    fn print3(a:i32) {
        println!("[{:?}] task cond #3 wait the cond {a} ", thread::current().id());
    }
```
