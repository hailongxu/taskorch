
use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp, TaskSubmitter};

mod util;
use util::*;

// Thread 1: Task execution (consumer) role
// Thread 2: Task generation (producer) role

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid1 = pool.insert_queue(&Queue::new()).unwrap();
    let submitter1 = pool.task_submitter(qid1).unwrap();
    // Step#4. start a thread and run
    pool.spawn_thread_for(qid1);

    // Step#3. create tasks
    consume_task_prompt(&submitter1);

    std::thread::spawn(||{
        produce_task(submitter1);
    });

    // Step#5. wait until all finished
    pool.join();
}

fn consume_task_prompt(submitter:&TaskSubmitter) {
    const Q:&'static str = "consumer";
    const PAD:&'static str = "  ";
    submitter.submit((||ff("comsumer", PAD, "init", "waiting task to do")).task());
}


fn produce_task(submitter:TaskSubmitter) {
    const Q:&'static str = "consumer";
    const PAD:&'static str = "  ";

    prompt("warn");
    submitter.submit((||ff(Q, PAD,"warn", "prepare to work.")).task());

    // Exit task with one condition
    const XN: &'static str = "X";
    let exit_task = |a:i32| exit_ff(Q, PAD,XN, a);
    prompt(XN);
    let id_exit = submitter.submit(exit_task.exit_task());

    const AN: &'static str = "Aadd";
    let task = move|a:i32,b:i32| ffadd(Q,PAD, AN, a, b, XN);
    prompt(AN);
    let id_add = submitter.submit(task.task().to(id_exit,0));

    let task = move||ffr(Q,PAD,"A1",(2,AN));
    prompt("A1");
    let _ = submitter.submit(task.task().to(id_add, 0));

    let task = move||ffr(Q,PAD,"A2",(3,AN));
    prompt("A2");
    let _ = submitter.submit(task.task().to(id_add, 1));
}

fn prompt(taskname:&'static str) {
    const Q:&'static str = "producer";
    sleep_millis!();
    let info = task_info!(Q);
    println!("{info}  submit task '{taskname}' to consumer.");
}
