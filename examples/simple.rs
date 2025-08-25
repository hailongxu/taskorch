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
            println!("task='A': fan to task=['B1','B2']");
            (10,"exit")
        })
        .bind_all_to((b1.input_ca::<0>(),b2.input_ca::<0>()));
    let _ = submitter.submit(b3);

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.join();
}
