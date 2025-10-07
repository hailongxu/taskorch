#![doc = include_str!("../README.md")]

use std::{
    collections::HashMap,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle}
};

#[macro_use]
mod log;

pub mod cond;
mod meta;
mod curry;
mod queue;
pub mod task;
mod submitter;

pub use cond::{
    CondAddr,TaskId,ArgIdx,Section
};

use queue::C1map;
pub use queue::{spawn_thread, Queue};

#[allow(deprecated)] // for TaskBuildOp will be removed at next ver.
pub use task::{
    Kind,
    TaskNeed,
    TaskBuildNew,TaskBuildOp,
    taskid_next,
};

pub use submitter::{TaskSubmitter,Submission,SummitResult,TaskSubmitError};


/// a handle to a thread spawned for queue
pub struct Jhandle(JoinHandle<()>,Arc<AtomicBool>);

impl Jhandle {
    /// record the thread handle into pool
    pub fn collect_into(self, pool:&mut Pool)->Option<usize> {
        pool.insert_thread_handle(self)
    }

    /// the thread exit once the current task complete
    pub fn exit_next(&mut self) {
        let _ = self.1.compare_exchange(
            false, true,
            Ordering::Acquire,Ordering::Relaxed);
    }

    /// block until the thread has exited
    pub fn join(self)->thread::Result<()> {
        self.0.join()
    }
}

/// Pool, a container that holds and managers all resources, such as threads and queues
pub struct Pool {
    queues: HashMap<usize,Queue>,
    jhands: HashMap<usize,Jhandle>,
    c1map: C1map,
    id_next: usize,
}

impl Pool {
    pub fn new()-> Self {
        log::init_starttime();
        warn!("Pool created.");
        Self {
            queues: HashMap::new(),
            jhands: HashMap::new(),
            c1map: C1map::new(),
            id_next: 0,
        }
    }

    /// A new ID for Queue.id and Thread.id not including Task.id
    fn next_id(&mut self)->usize {
        self.id_next += 1;
        self.id_next
    }

    /// creates a TaskSubmitter for the specified queue ID
    /// returns None if the queue ID does not exist
    pub fn task_submitter(&self, qid:usize)->Option<TaskSubmitter> {
        let queue = self.queues.get(&qid)?.clone();
        let c1map = self.c1map.clone();
        TaskSubmitter {qid, queue, c1map}.into()
    }

    /// gets the ref to Queue by ID
    pub fn queue(&self, qid:usize)->Option<&Queue> {
        self.queues.get(&qid)
    }

    /// gets the ref to thread handle by ID
    pub fn jhandle(&self, tid:usize)->Option<&Jhandle> {
        self.jhands.get(&tid)
    }

    /// returns the queue ID recorded in pool
    pub fn insert_queue(&mut self,queue:&Queue)->Option<usize> {
        let id = self.next_id();
        // update the queue
        let _r = self.queues.insert(id, queue.clone());
        debug!("Q#{id} created.");
        Some(id)
    }

    /// return thread.id in pool
    pub fn spawn_thread_for(&mut self, qid:usize)->Option<usize> {
        let Some(queue) = self.queue(qid) else {
            error!("Q#{qid} does not exist; thread starting is not allowed.");
            return None;
        };
        spawn_thread(queue).collect_into(self)
    }

    fn insert_thread_handle(&mut self, jhandle:Jhandle)->Option<usize> {
        let id = self.next_id();
        self.jhands.insert(id, jhandle)
            .map(|_|id)
    }

    #[allow(dead_code)]
    fn exit_next(&mut self, tid:usize)->Option<()> {
        let jhandle = self.jhands.get_mut(&tid)?;
        jhandle.exit_next();
        Some(())
    }

    /// Notifies each thread to exit upon completing each current running task.
    pub fn exit_next_all(&mut self) {
        for jhand in self.jhands.values_mut() {
            jhand.exit_next();
        }
    }

    /// block until all threads have exited
    pub fn join(self) {
        let thcount = self.jhands.len();
        let mut threadid_list_log = String::with_capacity(thcount*"thread(123) ".len());
        for (_innerid,handle) in self.jhands {
            let thid = handle.0.thread().id();
            if let Err(err) = handle.0.join() {
                let err = if let Some(s) = err.downcast_ref::<&str>() {
                    format!("thread panic: {}", s)
                } else if let Some(s) = err.downcast_ref::<String>() {
                    format!("thead panic: {}", s)
                } else {
                    format!("thread panic (unknow)")
                };
                error!("{err}");
                panic!("{}", err);
            }
            let thidstr = format!("{:?} ",thid);
            threadid_list_log.push_str(&thidstr);
            info!("pool received normal exit from {thid:?}.");
        }
        info!("pool with {thcount} threads: [{threadid_list_log}] exited ok.");
    }
}

