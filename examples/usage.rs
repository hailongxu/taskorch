
use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp, TaskSubmitter};

mod debug;
use debug::*;

/// construct Q1 with 2 thread
/// construct Q2 with 1 thread
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid1 = pool.insert_queue(&Queue::new()).unwrap();
    let submitter1 = pool.task_submitter(qid1).unwrap();
    let qid2 = pool.insert_queue(&Queue::new()).unwrap();
    let submitter2 = pool.task_submitter(qid2).unwrap();

    // Step#3. create tasks
    add_task_to_q1_by(&submitter1);
    add_task_to_q2_by(&submitter2);

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid1);
    pool.spawn_thread_for(qid1);
    pool.spawn_thread_for(qid2);

    // Step#5. wait until all finished
    pool.join();
}

fn add_task_to_q1_by(submitter:&TaskSubmitter) {
    let hello = String::from("hello");
    let task_hello = (||{
            let ti = ti!("Q#(1)");
            let hello = hello;
            println!("{ti}  task free Fn {hello}");
        }).task();
    submitter.submit(task_hello);

    let task_print1 = print1.task();
    submitter.submit(task_print1);

    // Exit task with one condition
    // This elegant approach ensures all threads exit one by one,
    // guaranteeing each thread can receive the exit message
    let id_exit = submitter.submit(exit2_on.exit_task());
    let id_exit = submitter.submit(exit1_on_to.exit_task().to(id_exit, 0));

    // Normal task with 8 conds and pass to exit task
    let task8 = print_on8_and_to.task().to(id_exit,0);
    let id8 = submitter.submit(task8);

    // below task pass cond to task id8
    submitter.submit((||{let ti=ti!("Q#(1)");w!();println!("{ti}  task pass cond [1] to id8");1}).task().to(id8,0));
    submitter.submit(task_hello_to.task().to(id8,1));
    submitter.submit(pass_to.task().to(id8,2));
    submitter.submit(pass_to.task().to(id8,3));
    submitter.submit(pass_to.task().to(id8,4));
    submitter.submit(pass_to.task().to(id8,5));
    submitter.submit(pass_to.task().to(id8,6));
    let id1 = submitter.submit(print_on1_and_to.task().to(id8,7));
    submitter.submit(task_to.task().to(id1,0));
}

fn add_task_to_q2_by(submitter:&TaskSubmitter) {
    let hello = String::from("hello");
    let task_hello = (||{
            w!(2);
            let ti = ti!("Q#(2)");
            let hello = hello;
            println!("{ti}  task free Fn {hello}");
        }).task();
    submitter.submit(task_hello);

    // Exit task with one condition
    let id_exit = submitter.submit(exit2_on.exit_task());

    // Normal task with 8 conds and pass to exit task
    let taskc2 = (|a:i32,b:i32| {
            w!();
            let r = a + b;
            let ti = ti!("Q#(2)");
            println!("{ti}  taskc2 recv ({a} {b}) and pass cond [{r}] to id_exit");
            r
    }).task().to(id_exit, 0);
    let c2 = submitter.submit(taskc2);

    // below task pass cond to task #c2
    fn ff(ci:usize)->i32 {
        let ti=ti!("Q#(2)");
        w!();
        println!("{ti}  task pass cond [1] to c2.#{ci}");
        1
    }

    let mut ci = 0;
    let cf = move||ff(ci);
    submitter.submit(cf.task().to(c2,ci));

    ci = 1;
    let cf = move||ff(ci);
    submitter.submit(cf.task().to(c2,ci));
}

// a free task
fn print1() {
    let ti = ti!("Q#(1)");
    println!("{ti}  task free fn");
}

// gen a cond i32
fn task_to()->i32 {
    let r = 5;
    let ti = ti!("Q#(1)");
    println!("{ti}  task pass cond [{r:?}] to task-id2");
    r
}

// gen a cond str
fn task_hello_to()->&'static str {
    let r = "hello";
    let ti = ti!("Q#(1)");
    println!("{ti}  task pass [{r:?}] to task-id8");
    r
}

// which accept a cond and gen a cond
fn print_on1_and_to(a:i32)->i32 {
    let r = 1;
    let ti = ti!("Q#(1)");
    println!("{ti}  task wait cond ({a}), and pass cond [{r}] to task-id8");
    assert_eq!(r,1);
    r
}

fn pass_to()->i32 {
    let ti = ti!("Q#(1)");
    w!();
    let r = 1;
    println!("{ti}  task pass cond [{r}] to task-id8");
    assert_eq!(r,1);
    r
}
// which accept a cond and gen a cond
fn print_on8_and_to(a:i32,b:&str,c:i32,d:i32,e:i32,f:i32,g:i32,h:i32)->i32 {
    let r = a+c+d+e+f+g+h;
    let ti = ti!("Q#(1)");
    println!("{ti}  task id8 , wait cond ({a},{b},{c},{d},{e},{f},{g},{h}), and pass [{r}] to exit task");
    assert_eq!(r, 7);
    r
}

// accept a cond and exit
fn exit1_on_to(a:i32)->i32 {
    let r = a+1;
    let ti = ti!("Q#(1)");
    println!("{ti}  exit task received the cond ({a}) pass cond [{r}] to exit2 and EXIT");
    r
}
fn exit2_on(a:i32)->i32 {
    let ti = ti!("Q#(1)");
    println!("{ti}  exit task received the cond ({a}) and EXIT");
    a+1
}
