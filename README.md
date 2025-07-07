# Task Orch

## System Composition
The entire concurrency library can generally be divided into four main components:

- **Task** — The minimal unit of execution, built from a fn or closure along with runtime metadata.
- **Queue** — The queue holds tasks that are waiting to be processed. It acts as a buffer where tasks are stored until they can be executed.
- **Threads** — Threads are responsible for executing the tasks retrieved from the queue.
- **Resource Pool** — Responsible for the lifecycle management of both the Queue and the Threads, establishing a mapping between names and instances.


## Task
### Task Modes
Tasks can be executed in **two distinct modes**:
- **Independent**: Runs freely without constraint.
- **Conditional**: Executes only when all conditions are satisfied.

### Task Flow
1. **Activation**:  
   A task is scheduled once all its required conditions are fulfilled.
2. **Execution**:  
   The task runs and computes its return value.
3. **Data Passing**:  
   If a `target anchor` is configured, the return value will be passed on to another task.

**Key Notes**:  
- **Conditions** correspond to the function’s parameters (0-indexed).

### Task ID Assignment
- **Explicit ID**: You can provide your own ID using a generator or by calling `taskid_next()`.
- **Auto-generated**: If you omit specifying an ID, the system will automatically assign one.

### Building a Task (3-Step Process)
1. **Prepare**:  Define a function or closure.
2. **Create**:  Create the task chained by `.task()`.
3. **Notify (Optional)**:  Chain tasks by calling `to()` to set a `target anchor`.  
   *This step can be skipped if the task does not produce any output.*

### Task Creation Code

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

let task_cc = (cc,1).task(); /// taskid is explicitly set to 1
let task_f0 = f0.task().to(1,0); // *** return type `i32` === cond #0 type `i32` ***
let task_f1 = f1.task().to(1,1); // *** return type `i8` === cond #1 type `i8`   ***
```

#### Note
NO `parameter`, NO `taskid` needed.  
NO `return`, NO `target anchor` required.  

**Case 1**:  [ **No** parameter, **No** return ]  
```rust
# use taskorch::TaskBuildNew as _;
let task = 
    (||{})          // <1> Define the body
        .task();    // <2> Create a task from the given closure.
                    // <3> `to()` skipped, as there is no return value
```

**Case 2**:  [ **No** parameter, **With** return ]  
```rust
# use taskorch::{TaskBuildNew as _, TaskBuildOp as _};
let task = 
    (||{3})         // <1> Define the body
        .task()     // <2> Create a task from the given closure.
        .to(2,0);   // <3> Set target;
                    //     the return value will be forwarded to task #2, condition #0.

let task = 
    (||{3})         // <1> ..
        .task();    // <2> ..
                    // <3> `to()` skipped, the return value dropped
```

**Case 3**:  [ **With** parameter, **No** return ]  
```rust
# use taskorch::TaskBuildNew as _;

let task = 
    (|_:i8|{})      // <1> Define the body, taskid is auto-generated.
        .task();    // <2> Create a task from the given closure.
                    // <3> `to()` skipped, as there is no return value
let task = 
    (|_:i8|{}, 1)   // <1> Define the body, with an explicit taskid.
        .task();    // <2> ..
                    // <3> ..
```

**Case 4**:  [ **With** parameter, **With** return ]  
```rust
# use taskorch::{TaskBuildNew as _, TaskBuildOp as _};

let task = 
    (|_:i8|{3})     // <1> Define the body, taskid is auto-generated.
        .task();    // <2> Create a task from the given closure.
                    // <3> `to()` skipped, the return value dropped
let task = 
    (|_:i8|{3}, 1)  // <1> Define the body, with an explicit taskid.
        .task();    // <2> ..
                    // <3> ..

let task = 
    (|_:i8|{3})     // <1> Define the body, taskid is auto-generated.
        .task()     // <2> Create a task from the given closure.
        .to(2,0);   // <3> Set target;
                    //     the return value will be forwarded to task #2 and cond #0

let task = 
    (|_:i8|{3}, 1) // <1> Define the body, with an explicit taskid.
        .task()    // <2> ..
        .to(2,0);  // <3> ..
```
**Exit task creation**  
The only difference here is the use of `.exit_task()` instead of `.task()`.

## ⚠️ API NOTE
As this project is currently in early active development, the API is **highly unstable** and **will change** in subsequent versions.

## Example
```rust
use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp};
// create 3 tasks and run 
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    let submitter = pool.task_submitter(qid).unwrap();

    // Step#3. create tasks

    // an indepent task
    submitter.submit((||println!("free-task:  Hello, 1 2 3 ..")).task());

    // an exit task with ONE str cond
    let id_exit = submitter.submit(
        (|msg:&str|
            println!("exit-task:  received ({msg:?}) and EXIT")
        ).exit_task()
    );

    // another task pass message to exit task
    submitter.submit(
        (||{
            const MSG: &'static str = "exit";
            println!("task 'AA':  I pass [\"{MSG}\"] to exit-task to exit");
            MSG
        }).task().to(id_exit, 0)
    );

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.join();
}
```
For a more complex demo, see the `usage` example.
