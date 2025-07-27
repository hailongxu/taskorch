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

### Task **<u>Inner</u>** Flow
1. **Activation**: A task is scheduled once all its required conditions are fulfilled.
2. **Execution**: The task runs and computes its return value.
3. **Data Passing**: If a `target anchor` is configured, the return value will be passed on to another task.  
see [Task creations code](#task-creation-code)

**Key Notes**:  
- **Conditions** correspond to the function’s parameters (0-indexed).


### task components
The task consists of two main parts:
1. **task body**, The core logic of the task, responsible for execution and producing a result.
2. **result distribution**, determine how the result is passed to other Anchors

### Task **<u>Inter</u>** Flow
#### 1. N --> 1
Pass each `[task1.result, task2.result, ..]` to `task(cond#1,cond#2,..)` using `.to()`  
see [Example task N->1](#task-n-1-creation-code).
#### 2. 1 --> N
Map `task.result` to `[task1.cond, task2.cond,..]` using `.fan_tuple_with()`  
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
2. **Create**:  Create the task from function or closure chained by `.into_task()`.
3. **Notify (Optional)**:  
    Forward task's result by calling `to()` to set a `target anchor`.  
    Or forward task's result by calling `fan_tuple_with()` to multi `target anchor`.   
   *This step can be skipped if the task does not produce any output.*

## Task Creation Code
#### Note
NO `parameter`, NO `taskid` needed.  
NO `return`, NO `target anchor` required.  

### Create a task body with `.into_task()`
Any function or closure by calling `.into_task()` will complete constructing a task;
```rust
# use taskorch::TaskBuildNew as _;
// with an explicit taskid = 1
let task = (|_:i32|{3}, 1).into_task();
// with no explicit taskid, the system will auto-generate one when submitting !!!!
let task = (|_:i32|{3}   ).into_task();
```
**Exit task creation**  
The only difference here is the use of `.into_exit_task()` instead of `.into_task()`.

## Task notify code
### Task result: N->1, pass to single task using `.to()`
```rust
# use taskorch::TaskBuildNew as _;
let task  = (|i16,i32|{}, 1).into_task();  // task#1 with 2 cond (#0 i16,#1 i32)
let task1 = (||{2}).into_task().to(1,0); // task1 -> task
let task2 = (||{3}).into_task().to(1,1); // task2 -> task
```

### Task result: 1->N, pass to multi task using `.fan_tuple_with()`
```rust
# use taskorch::TaskBuildNew as _;
let task1 = (|_:i16|{3}, 1).into_task(); // task#1 with cond#0
let task2 = (|_:i32|{3}, 2).into_task(); // task#2 with cond#0
let task  = (||(2i16,3i32)).into_task()  // task return type (#0 i16, #1 i32)
            .fan_tuple_with(|(a,b):(i16,i32)|( // input parameter is the return type
                (a,Anchor(1,0)), // a --> task#1.cond#0
                (b,Anchor(2,0)), // b --> task#2.cond#0
            ));
```


#### ⚠️ Type cast NOTE
> **❗ Error-prone operation!**  
> When forwarding a task result to a conditional task's condition point:  
> - **Ensure** the result type **must be identical to** the condition type.  
> - **Violation will trigger `panic`!**  
```rust
# use taskorch::{TaskBuildNew as _, TaskBuildOp as _};
// Sample code explanation
let cc = |a:i32,b:i8|{}; // the type of cond #0 is `i32`
                         // the type of cond #1 is `i8` 
let f0 = ||5i32; // the return type is `i32`
let f1 = ||5i8;  // the return type is `i8`

let task_cc = (cc,1).into_task(); // taskid is explicitly set to 1
let task_f0 = f0.into_task().to(1,0); // *** return type `i32` === cond #0 type `i32` ***
let task_f1 = f1.into_task().to(1,1); // *** return type `i8` === cond #1 type `i8`   ***
```


## ⚠️ API NOTE
As this project is currently in early active development, the API is **highly unstable** and **will change** in subsequent versions.

## Example
```rust
use taskorch::{Anchor, Pool, Queue, TaskBuildNew};
//  A       => [B1, B2] ## 1->N
// [B1, B2] =>  Exit    ## N->1
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    let submitter = pool.task_submitter(qid).unwrap();

    // Step#3. create tasks

    // an indepent task
    submitter.submit((||println!("free-task:  Hello, 1 2 3 ..")).into_task());

    // an exit task with cond(#0 i32, #2 str)
    let id_exit = submitter.submit(
        (|a:i32,msg:&str|
            println!("exit-task:  received ({a},{msg:?}) and EXIT")
        ).into_exit_task()
    );

    // N->1 : pass i32 to exit-task
    let id_b1 = submitter.submit(
        (|a:i32|{println!("task 'B1':  pass ['{a}'] to exit-task"); a})
        .into_task().to(id_exit, 0)
    );

    // N->1 : pass str to exit task
    let id_b2 = submitter.submit(
        (|msg:&'static str|{println!("task 'B2':  pass ['{msg}'] to exit-task");msg})
        .into_task().to(id_exit, 1)
    );

    // 1->N : map result to task-b1 and task-b2
    submitter.submit((||3).into_task().fan_tuple_with(move|a: i32|{
        println!("task 'A': fan to 'B1' 'B2'");
        ((a,Anchor(id_b1,0)),("exit",Anchor(id_b2,0)))
    }));

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
