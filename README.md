taskorch
===

## System Composition
The entire concurrency library can generally be divided into four main components:

- **Task** — The minimal unit of execution, built from a fn or closure along with runtime metadata.
- **Queue** — The queue holds tasks that are waiting to be processed. It acts as a buffer where tasks are stored until they can be executed.
- **Threads** — Threads are responsible for executing the tasks retrieved from the queue.
- **Resource Pool** — Responsible for the lifecycle management of both the Queue and the Threads, establishing a mapping between IDs and instances.


## Task
### Task Modes
Tasks can be executed in **two distinct modes**:
- **Independent**: Runs freely without constraint.
- **Conditional**: Executes only when all conditions are satisfied.

### task components
The task consists of two main parts:
1. **Result generation**: The core logic responsible for executing the task and producing a result.
2. **Result distribution**: Determines how the result is delivered to other tasks identified by `CondAddr`.

### Task **Inner** Flow
1. **Activation**: A task is scheduled once all its required conditions are fulfilled.
2. **Execution**: The task runs and output single or multiple results.
3. **Data Passing**: If a `target condaddr` is configured, the return value will be forwarded to the specified task.  
see [Task creations code](#task-creation-code)

**Key Notes**:  
- **Conditions** correspond to the function’s parameters (0-indexed).

### Task **Inter** Flow
#### 1. N --> 1
Results from multi tasks `[task1.result, task2.result, ..]` are passed as inputs to a single `task(cond#1,cond#2,..)` using `.bind_to()`  
see [Example task N->1](#task-n-1-creation-code).
#### 2. 1 --> N
The result from a single `task.result` is distributed to the conditions of multiple tasks `[task1.cond, task2.cond,..]` using `.bind_all_to()`  
see [Example task 1->N](#task-1-n-creation-code) (since v0.3.0).


### Task ID Assignment
- **Explicit ID**: You can provide your own ID using a generator or by calling `taskid_next()`.
- **Auto-generated**: If you omit specifying an ID, the system will automatically assign one.

### Usage
Add only one of the following lines to your `Cargo.toml`:
```toml
# No logs, no color
taskorch = "0.3.0"

# With logging, and colored output
taskorch = {version="0.3.0", features=["log-info", "log-color"]}
```
Optional features can be enabled based on your needs (see [Available Features](#available-features)).

### Building a Task (2-Step Process)
1. **Create**:  Convert a function or closure into a task by calling `.into_task()`.
2. **Notify (Optional)**:  
    - Use `bind_to()` to send the task result to a single `target condaddr`.  
    - Use `bind_all_to()` to distribute the multi-result from single task to multi `target condaddr`.   
    > ⚠️ **Prerequisite**: Call `.map_tuple_with()` first to map the single result into multiple outputs.

*The second step is optional and can be skipped if the task does not produce any result.*

## Task Creation Code
#### Note
NO `parameter`, NO `taskid` needed.  
NO `return`, NO `target condaddr` required.  

### Create a task body with `.into_task()`
Any function or closure by calling `.into_task()` will complete constructing a task;
```rust
# use taskorch::{TaskBuildNew,TaskId};
// with an explicit taskid = 1
let task = (|_:i32|{3}, TaskId::from(1)).into_task();
// with no explicit taskid, the system will auto-generate one when submitting !!!!
let task = (|_:i32|{3}   ).into_task();
```
**Exit task creation**  
The only difference here is the use of `.into_exit_task()` instead of `.into_task()`.

## Task notify code
### Task result: N->1, pass to single task using `.bind_to()`
```rust
# use taskorch::{TaskBuildNew,TaskId};
let task  = (|i16,i32|{}, TaskId::from(1)).into_task();  // task#1 with 2 cond (#0 i16,#1 i32)
let task1 = (||{2}).into_task().bind_to(task.input_ca::<0>()); // task1 -> task
let task2 = (||{3}).into_task().bind_to(task.input_ca::<1>()); // task2 -> task
```

### Task result: 1->N, pass to multi task using `.bind_all_to()`
```rust
# use taskorch::{TaskBuildNew,TaskId};
let task1 = (|_:i16|{3}, TaskId::from(1)).into_task(); // task#1 with cond<i16>#0
let task2 = (|_:i32|{3}, TaskId::from(2)).into_task(); // task#2 with cond<i32>#0
let task = (|| 2i16).into_task() // Task output: i16
    .map_tuple_with(|a: i16| (1i16, 2i32)) // Transforms input into multiple outputs
    // Output binding:
    // - result.#0: i16 (bound to task1.input<0>)
    // - result.#1: i32 (bound to task2.input<0>)
    // Type safety: All bindings are verified at compile time
    .bind_all_to((task1.input_ca::<0>(), task2.input_ca::<0>()));
```

#### ⚠️ Type Safety Note
Starting with v0.3, ***type checking** for binding **outputs to inputs** is enforced at **compile time** rather than at runtime as in previous versions. This shift enables early error detection during development, and reduces debugging time.

## ⚠️ API NOTE
As this project is currently in early active development, the API is **highly unstable** and **will change** in subsequent versions.

## Example
```rust
use taskorch::{Pool, Queue, TaskBuildNew};
// [A]      => [B1, B2] ## 1->N
// [B1, B2] => [Exit]   ## N->1
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    let submitter = pool.task_submitter(qid).unwrap();

    // Step#3. create tasks

    // an indepent task
    let task = (||println!("task='free':  Hello, 1 2 3 ..")).into_task();
    let _ = submitter.submit(task);

    // an exit task with cond(#0 i32, #2 str)
    let exit = submitter.submit(
        (|a:i32,msg:&str|
            println!("task='exit': received ({a},{msg:?}) and EXIT")
        ).into_exit_task()
    ).unwrap();

    // N->1 : pass i32 to exit-task.p0
    let b1 = (|a:i32|{println!("task='B1':  pass ['{a}'] to task='exit'"); a})
        .into_task()
        .bind_to(exit.input_ca::<0>());
    let b1 = submitter.submit(b1).unwrap();

    // N->1 : pass str to exit task.p1
    let b2 = (|msg:&'static str|{println!("task='B2':  pass ['{msg}'] to task='exit'");msg})
        .into_task()
        .bind_to(exit.input_ca::<1>());
    let b2 = submitter.submit(b2).unwrap();

    // 1->N : map result to task-b1 and task-b2
    let b3 = (||())
        .into_task()
        .map_tuple_with(move|_: ()|{
            println!("task='A': map `()=>(i32,&str)` and pass to task=['B1','B2']");
            (10,"exit")
        })
        .bind_all_to((b1.input_ca::<0>(),b2.input_ca::<0>()));
    let _ = submitter.submit(b3);

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.join();
}
```
For a more complex demo, see the `usage` and `spsc` example.
```shell
cargo run --example usage --features="log-trace,log-color"
cargo run --example spsc --features="log-trace,log-color"
```


## Available Features
All logs are compile-time controlled and have zero runtime overhead when disabled.  

- **`log-error`**: Logs ERROR level only  
- **`log-warn`**: Logs WARN level and above  
- **`log-info`**: Logs INFO level and above  
- **`log-debug`**: Logs DEBUG level and above  
- **`log-trace`**: Logs TRACE level and above (most verbose)  
- **`log-color`**: Adds ANSI color to log messages in the terminal  

> ⚠️ Note:   
These log level features are mutually exclusive - only one or none can be enabled at a time.  
**No logs are emitted by default.**  
**Color is disabled by default.**  

### 🕒 Timestamp Format in Logs
The timestamp used in logs is measured from the earliest of the following events:
- The time when the **first log message was emitted**
- The time when the **first `Pool`** was created

This is a **relative time** (not absolute wall-clock time), designed for analyzing task sequences.
