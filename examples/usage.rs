use taskorch::{TaskCurrier, Pool, Queue, Anchor, TaskBuildNew, TaskBuildOp};

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();

    // Step#3. create tasks

    // task#1. add a free task closure
    let hello = String::from("hello");
    let task_hello = TaskCurrier::new(||{ let hello = hello; println!("task free Fn {hello}");});
    pool.add(task_hello);

    // task#2. add a free task function
    let task_print1 = TaskCurrier::new(print1);
    pool.add(task_print1);

    // task#3. add a task with cond
    let id_exit = pool.add(TaskCurrier::new(exit_on).exit());

    // task#4. add a task which get cond and gen cond to <task.id_exit>
    let task2 = TaskCurrier::new(print2_on_and_to).to(Anchor(id_exit,0));
    let id2 = pool.add(task2);

    // task#5. add a task which get cond and gen cond to <task.id2>
    let id = pool.add(TaskCurrier::new(print_on_and_to).to(Anchor(id2,0)));
    
    // task#6. add a task which gen cond to <task.id>
    pool.add(TaskCurrier::new(task_to).to(Anchor(id,0)));

      // task#7. add a task which gen cond to <task.id2>
    pool.add(TaskCurrier::new(task_hello_to).to(Anchor(id2,1)));
  
    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.wait();
}

// a free task
fn print1() {
    println!("task free fn");
}

// gen a cond i32
fn task_to()->i32 {
    let r = 5;
    println!("task to => r.cond={r:?}");
    r
}

// gen a cond str
fn task_hello_to()->&'static str {
    let r = "hello";
    println!("task to => r.cond={r:?}");
    r
}

// which accept a cond and gen a cond
fn print_on_and_to(a:i32)->i32 {
    let r = a * 2;
    println!("task on-to #3 , wait cond {a}, and => r.cond={r}");
    r
}

// which accept a cond and gen a cond
fn print2_on_and_to(a:i32,b:&str)->i32 {
    let r = a + a;
    println!("task on-to #3 , wait cond ({a}, {b}), and => r.cond={r}");
    r
}

// accept a cond and exit
fn exit_on(a:i32) {
    println!("exit task #4 wait the cond {a} ");
}
