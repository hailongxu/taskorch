use taskorch::{CondAddr, Pi, Pool, Queue, TaskBuildNew};
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
    let _ = submitter.submit((||println!("task='free':  Hello, 1 2 3 ..")).into_task());

    // an exit task with cond(#0 i32, #2 str)
    let id_exit = submitter.submit(
        (|a:i32,msg:&str|
            println!("task='exit': received ({a},{msg:?}) and EXIT")
        ).into_exit_task()
    ).unwrap();

    // N->1 : pass i32 to exit-task.p0
    let id_b1 = submitter.submit(
        (|a:i32|{println!("task='B1':  pass ['{a}'] to task='exit'"); a})
        .into_task().to(((id_exit, Pi::PI0)).into())
    ).unwrap();

    // N->1 : pass str to exit task.p1
    let id_b2 = submitter.submit(
        (|msg:&'static str|{println!("task='B2':  pass ['{msg}'] to task='exit'");msg})
        .into_task().to((id_exit, Pi::PI1).into())
    ).unwrap();

    // 1->N : map result to task-b1 and task-b2
    let _ = submitter.submit((||3).into_task().fan_tuple_with(move|a: i32|{
        println!("task='A': fan to task=['B1','B2']");
        ((a,(id_b1,Pi::PI0).into()),("exit",(id_b2,Pi::PI0).into()))
    }));

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.join();
}
