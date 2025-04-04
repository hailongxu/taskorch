use std::{
    any::Any,
    collections::HashMap,
    sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc},
    thread::{self, JoinHandle}
};

mod curry;
mod task;
mod queue;
use queue::{when_c1_comed, C1map};
pub use queue::{spawn_thread, Queue};
use task::NormalTask;
pub use task::Which;


pub(crate) static TASK_ID:TaskIdGen = TaskIdGen::new();

struct TaskIdGen {
    nexter: AtomicUsize
}
impl TaskIdGen {
    const fn new()->Self {
        Self {
            nexter: AtomicUsize::new(0)
        }
    }
    fn next(&self)->usize {
        self.nexter.fetch_add(1, Ordering::Relaxed)
    } 
}


pub struct Jhandle(JoinHandle<()>,Arc<AtomicBool>);

impl Jhandle {
    pub fn collect_into(self, pool:&mut Pool)->Option<usize> {
        pool.insert_thread(self)
    }

    pub fn exit_next(&mut self) {
        let _ = self.1.compare_exchange(
            false, true,
            Ordering::Acquire,Ordering::Relaxed);
    }

    pub fn wait(self)->thread::Result<()> {
        self.0.join()
    }
}

pub struct Pool {
    queues: HashMap<usize,Queue>,
    jhands: HashMap<usize,Jhandle>,
    // c1queue: Queue,
    c1map: C1map,
    id_next: usize,
}

impl Pool {
    pub fn new()-> Self {
        Self {
            queues: HashMap::new(),
            jhands: HashMap::new(),
            // c1queue: Queue::new(),
            c1map: C1map::new(),
            id_next: 0,
        }
    }

    fn next_id(&mut self)->usize {
        self.id_next += 1;
        self.id_next
    }

    pub fn add<F,R>(&self,f:F,which:Which)
        where
        F:Fn()->R+Send+'static,
        R:Send+'static
    {
        let c1map = self.c1map.clone();
        // let c1queue = self.c1queue.clone();
        let c1queue = self.queues.values().next().unwrap().clone();
        let postdo = move |r: Box<dyn Any>| {
            let Ok(r) = r.downcast::<R>() else {
                assert!(false,"failed to conver to R type");
                return;
            };
            let r: R = *r;
            when_c1_comed(&which, r, c1map.clone(), c1queue);
        };
        let postdo = Box::new(postdo);
        let (_,normal) = self.queues.iter().next().unwrap();
        normal.add(f,&which,postdo);
    }

    pub fn addc1<F,P1,R>(&self,f:F,which:Which)->usize
        where
        F:Fn(P1)->R+Send+'static,
        P1:Send+Clone+'static,
        R:Send+'static
    {
        let c1map = self.c1map.clone();
        // let c1queue = self.c1queue.clone();
        let c1queue = self.queues.values().next().unwrap().clone();
        let postdo = move |r: Box<dyn Any>| {
            let Ok(r) = r.downcast::<R>() else {
                assert!(false,"failed to conver to R type");
                return;
            };
            let r: R = *r;
            when_c1_comed(&which, r, c1map.clone(), c1queue);
        };
        let postdo = Box::new(postdo);
        let task = NormalTask::from(f);
        let id = self.c1map.insert(task, postdo).unwrap();
        id
    }

    pub fn add_exit(&self, f:impl Fn()+'static+Send) {
        let (_,normal) = self.queues.iter().next().unwrap();
        normal.add_exit(f);
    }

    fn queue(&self, qid:usize)->Option<&Queue> {
        self.queues.get(&qid)
    }

    fn jhandle(&self, tid:usize)->Option<&Jhandle> {
        self.jhands.get(&tid)
    }

    pub fn insert_queue(&mut self,queue:&Queue)->Option<usize> {
        let id = self.next_id();
        let r = self.queues.insert(id, queue.clone());
        dbg!(r.is_some());
        Some(id)
    }

    pub fn insert_thread_from(&mut self, qid:usize)->Option<usize> {
        let queue = self.queue(qid)?;
        spawn_thread(queue).collect_into(self)
    }

    fn insert_thread(&mut self, jhandle:Jhandle)->Option<usize> {
        let id = self.next_id();
        self.jhands.insert(id, jhandle)
            .map(|_|id)
    }

    fn exit_next(&mut self, tid:usize)->Option<()> {
        let jhandle = self.jhands.get_mut(&tid)?;
        jhandle.exit_next();
        Some(())
    }

    fn exit_next_all(&mut self) {
        for jhand in self.jhands.values_mut() {
            jhand.exit_next();
        }
    }

    pub fn wait(self) {
        for handle in self.jhands {
            handle.1.0.join().unwrap();
        }
    }
}
