use taskorch::{Pool, Queue, TaskCurrier, Kind, Which};

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();

    // Step#3. create tasks

    // task#1. add a indepent task
    pool.add(TaskCurrier::from(||println!("task free Fn say hello")));

    // task#3. add a task with cond
    let id_exit = pool.add(TaskCurrier::from((|msg:&str|println!("recv '{msg}' and exit"),Kind::Exit)));

    // let id_exit = &id_exit;
    pool.add(TaskCurrier::from((||{println!("gen a cond 'msg:exit' ");"msg:exit"},Which::new(id_exit, 0))));

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.wait();
}
