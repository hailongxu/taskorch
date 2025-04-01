use std::{
    collections::{HashMap, VecDeque},
    sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex},
    thread::{self, JoinHandle}
};

mod curry;
mod task;
use task::*;

#[derive(Clone)]
pub struct Queue(Arc<(Mutex<VecDeque<Box<dyn Task<R=()>+Send>>>,Condvar)>);

impl Queue {
    pub fn new()->Self {
        Queue(Arc::new((Mutex::new(VecDeque::new()),Condvar::new())))
    }

    pub fn add<F,R>(&mut self,f:F)
        where
        F:Fn()->R + Send+'static,
        R: Send+'static,
    {
        let task = NormalTask::from(f);
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back(Box::new(task));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn add_exit(&mut self, f:impl Fn()+'static+Send) {
        let exit_task = ExitTask::from(f);
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back(Box::new(exit_task));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn pop(&mut self)->Option<Box<dyn Task<R=()>+Send>> {
        self
            .0
            .0
            .lock()
            .unwrap()
            .pop_front()
    }
    
    pub fn clear(&mut self) {
        self
            .0
            .0
            .lock()
            .unwrap()
            .clear()
    }

    pub fn len(&self)->usize {
        self
            .0
            .0
            .lock()
            .unwrap()
            .len()
    }
}

pub fn spawn_thread(queue:&Queue)-> Jhandle {
    let quit_flag = Arc::<AtomicBool>::new(AtomicBool::new(false));
    let quit = quit_flag.clone();
    let queue = queue.0.clone();
    let handle = thread::spawn(move||{
        loop {
            if quit.load(Ordering::Relaxed) {
                break;
            }
            
            let mut m = queue.0.lock().unwrap();
            let task = m.pop_front();
            if let Some(task) = task {
                drop(m);
                task.call();
                if let TaskKind::Exit = task.kind() {
                    break;
                }
            } else {
                let _unused = queue.1.wait(m);
            }
        }
    });
    Jhandle(handle,quit_flag)
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
    id_next: usize,
}

impl Pool {
    pub fn new()-> Self {
        Self {
            queues: HashMap::new(),
            jhands: HashMap::new(),
            id_next: 0,
        }
    }

    fn next_id(&mut self)->usize {
        self.id_next += 1;
        self.id_next
    }

    fn queue(&self, qid:usize)->Option<&Queue> {
        self.queues.get(&qid)
    }

    fn jhandle(&self, tid:usize)->Option<&Jhandle> {
        self.jhands.get(&tid)
    }

    pub fn insert_queue(&mut self,queue:&Queue)->Option<usize> {
        let id = self.next_id();
        self.queues.insert(id, queue.clone())
            .map(|_|id)
    }

    fn insert_thread_from(&mut self, qid:usize)->Option<usize> {
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
