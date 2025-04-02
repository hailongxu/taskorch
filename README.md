# concurrent-pool
The entire concurrency library can generally be divided into three main components:

- **Queue** — The queue holds tasks that are waiting to be processed. It acts as a buffer where tasks are stored until they can be executed.
- **Threads** — Threads are responsible for executing the tasks retrieved from the queue.
- **Resource Pool** — Responsible for the lifecycle management of both the Queue and the Threads, establishing a mapping between names and instances.

## example
```rust
    use concurrent as coc;
    let mut queue = coc::Queue::new();
    queue.add(print1);
    queue.add(||print2(&33));
    queue.add(||println!("task #3"));
    queue.add_exit(||println!("exit, after this"));
    coc::spawn_thread(&queue).wait().unwrap();
    fn print1() {
        println!("[{:?}] task #1 ", thread::current().id());
    }
    fn print2(a:&i32) {
        println!("[{:?}] task #2 {a} ", thread::current().id());
    }

```