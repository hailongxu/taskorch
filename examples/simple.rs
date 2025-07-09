use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp};

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    let submitter = pool.task_submitter(qid).unwrap();

    // Step#3. create tasks

    // an indepent task
    submitter.submit((||println!("task free say hello")).task());

    // an exit task with ONE str cond
    let id_exit = submitter.submit(
        (
            |msg:&str|println!("exit task, recved ({msg:?}) and EXIT"),
            1
        ).exit_task()
    );
    assert_eq!(id_exit,1);

    // normal task pass message to exit task
    submitter.submit(
        (move||{
            let id_exit = &id_exit;
            println!("normal pass [\"msg:exit\"] to: task#{id_exit}");
            "msg:exit"
        }).task().to(id_exit, 0)
    );

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.join();
}
