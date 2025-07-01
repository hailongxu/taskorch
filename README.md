# Task Orch
Tasks can be orchestrated in three distinct modes — independent, dependent, and conditional — and executed concurrently to complete the full workload.

The entire concurrency library can generally be divided into three main components:

- **Queue** — The queue holds tasks that are waiting to be processed. It acts as a buffer where tasks are stored until they can be executed.
- **Threads** — Threads are responsible for executing the tasks retrieved from the queue.
- **Resource Pool** — Responsible for the lifecycle management of both the Queue and the Threads, establishing a mapping between names and instances.

As this is the initial development release (v0.1.0), the API is **highly unstable** and **will change** in subsequent versions.

## Example
```rust
use taskorch::{Pool, Queue, TaskCurrier, Kind, Which};

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();

    // Step#3. create tasks

    // task#1. add a indepent task
    pool.add(TaskCurrier::from(||println!("task free Fn say hello")));

    // task#2. add a exit task with cond
    let id_exit = pool.add(TaskCurrier::from((
        |msg:&str|println!("this is exit task, recved '{msg}' and exit"),
        Kind::Exit
    )));

    // task#3. gen a task message to notify exit task to be scheduled
    pool.add(TaskCurrier::from((
        move||{
            let id_exit = &id_exit;
            println!("gen a cond 'msg:exit' ref out value:{id_exit} ");
            "msg:exit"
        },
        Which::new(id_exit, 0)
    )));

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.wait();
}
```
