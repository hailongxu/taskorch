use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp};

mod debug;
use debug::*;

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();

    // Step#3. create tasks
    let hello = String::from("hello");
    let task_hello = (||{
            let ti = ti!();
            let hello = hello;
            println!("{ti}  task free Fn {hello}");
        }).task();
    pool.add(task_hello);

    let task_print1 = print1.task();
    pool.add(task_print1);

    // Exit task with one condition
    // This elegant approach ensures all threads exit one by one,
    // guaranteeing each thread can receive the exit message
    let id_exit = pool.add(exit2_on.exit_task());
    let id_exit = pool.add(exit1_on_to.exit_task().to(id_exit, 0));

    // Normal task with 8 conds and pass to exit task
    let task8 = print_on8_and_to.task().to(id_exit,0);
    let id8 = pool.add(task8);

    // below task pass cond to task id8
    pool.add((||{let ti=ti!();w!();println!("{ti}  task pass cond [1] to id8");1}).task().to(id8,0));
    pool.add(task_hello_to.task().to(id8,1));
    pool.add(pass_to.task().to(id8,2));
    pool.add(pass_to.task().to(id8,3));
    pool.add(pass_to.task().to(id8,4));
    pool.add(pass_to.task().to(id8,5));
    pool.add(pass_to.task().to(id8,6));
    let id1 = pool.add(print_on1_and_to.task().to(id8,7));
    pool.add(task_to.task().to(id1,0));

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.wait();
}

// a free task
fn print1() {
    let ti = ti!();
    println!("{ti}  task free fn");
}

// gen a cond i32
fn task_to()->i32 {
    let r = 5;
    let ti = ti!();
    println!("{ti}  task pass cond [{r:?}] to task-id2");
    r
}

// gen a cond str
fn task_hello_to()->&'static str {
    let r = "hello";
    let ti = ti!();
    println!("{ti}  task pass [{r:?}] to task-id8");
    r
}

// which accept a cond and gen a cond
fn print_on1_and_to(a:i32)->i32 {
    let r = 1;
    let ti = ti!();
    println!("{ti}  task wait cond ({a}), and pass cond [{r}] to task-id8");
    assert_eq!(r,1);
    r
}

fn pass_to()->i32 {
    let ti = ti!();
    w!();
    let r = 1;
    println!("{ti}  task pass cond [{r}] to task-id8");
    assert_eq!(r,1);
    r
}
// which accept a cond and gen a cond
fn print_on8_and_to(a:i32,b:&str,c:i32,d:i32,e:i32,f:i32,g:i32,h:i32)->i32 {
    let r = a+c+d+e+f+g+h;
    let ti = ti!();
    println!("{ti}  task id8 , wait cond ({a},{b},{c},{d},{e},{f},{g},{h}), and pass [{r}] to exit task");
    assert_eq!(r, 7);
    r
}

// accept a cond and exit
fn exit1_on_to(a:i32)->i32 {
    let r = a+1;
    let ti = ti!();
    println!("{ti}  exit task received the cond ({a}) pass cond [{r}] to exit2 and EXIT");
    r
}
fn exit2_on(a:i32)->i32 {
    let ti = ti!();
    println!("{ti}  exit task received the cond ({a}) and EXIT");
    a+1
}
