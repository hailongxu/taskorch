
use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp, TaskSubmitter};

mod util;
use util::*;

/// construct Q1 with 2 thread
/// construct Q2 with 1 thread
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid1 = pool.insert_queue(&Queue::new()).unwrap();
    let qid2 = pool.insert_queue(&Queue::new()).unwrap();
    let submitter1 = pool.task_submitter(qid1).unwrap();
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
    const Q:&'static str = "Q#A";
    const PAD:&'static str = "";
    
    submitter.submit((||ff(Q, PAD,"A-free","Hi, I am free.")).task());

    // Exit task construction
    // This elegant approach ensures all threads exit one by one,
    // guaranteeing each thread can receive the exit message
    let exit_task = |a:i32| exit_ff(Q,PAD,"Z1", a);
    let id_exit = submitter.submit(exit_task.exit_task());
    let exit_task = move|a:i32| exit_ffpr(Q, PAD,"Z2",a,"Z1");
    let id_exit = submitter.submit(exit_task.exit_task().to(id_exit,0));

    let task = move|a:i32,b:i32| ffadd(Q, PAD,"Aadd", a, b, "Z2");
    let id_add = submitter.submit(task.task().to(id_exit,0));

    let task = move||ffr(Q,PAD,"A1",(2,"Aadd"));
    let _ = submitter.submit(task.task().to(id_add, 0));

    let task = move||ffr(Q,PAD,"A2",(3,"Aadd"));
    let _ = submitter.submit(task.task().to(id_add, 1));
}

fn add_task_to_q2_by(submitter:&TaskSubmitter) {
    const Q:&'static str = "Q#B";
    const PAD:&'static str = "";

    submitter.submit((||ff(Q, PAD,"B-free", "Hi, I am free too.")).task());

    // Exit task with one condition
    let exit_task = |a:i32| exit_ff(Q, PAD,"Y1", a);
    let id_exit = submitter.submit(exit_task.exit_task());

    let task = move|a:i32| ffpr(Q,PAD, "B1", a, (3, "Y1"));
    let taskid = submitter.submit(task.task().to(id_exit,0));

    let task = move|a:i32| ffpr(Q,PAD, "B2", a, (4, "B1"));
    let taskid = submitter.submit(task.task().to(taskid, 0));

    let task = move||ffr(Q,PAD,"B3",(4,"B2"));
    let _ = submitter.submit(task.task().to(taskid,0));
}

