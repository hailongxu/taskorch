use std::{
    collections::{HashMap, VecDeque},
    sync::{atomic::{AtomicBool, Ordering},Arc, Condvar, Mutex},
    thread
};

use crate::{task::{ExitTask, NormalTask, Task, TaskKind}, Jhandle};


#[derive(Clone)]
pub struct Queue(Arc<(Mutex<VecDeque<Box<dyn Task<R=()>+Send>>>,Condvar)>);

impl Queue {
    pub fn new()->Self {
        Queue(Arc::new((Mutex::new(VecDeque::new()),Condvar::new())))
    }

    pub fn add_boxtask(&self,task:Box<dyn Task<R=()>+Send+'static>) {
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back(task);
        if is_empty {
            self.0.1.notify_one();
        }
    }
    pub fn add<F,R>(&self,f:F)
        where
        F:Fn()->R + Send+'static,
        R: Send+'static,
    {
        let task = NormalTask::from(f);
        let task = Box::new(task);
        self.add_boxtask(task);
    }

    pub fn add_exit(&self, f:impl Fn()+'static+Send) {
        let exit_task = ExitTask::from(f);
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back(Box::new(exit_task));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn pop(&self)->Option<Box<dyn Task<R=()>+Send>> {
        self
            .0
            .0
            .lock()
            .unwrap()
            .pop_front()
    }
    
    pub fn clear(&self) {
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


#[derive(Clone)]
pub struct C1map(Arc<(Mutex<HashMap<usize,Box<dyn Task<R=()>+Send>>>,Condvar)>);
impl C1map {
    fn new()->Self {
        Self(
            Arc::new((Mutex::new(HashMap::new()),Condvar::new()))
        )
    }
    fn insert<T>(&self,task: T)->Option<usize>
    where T: Task<R=()> + Send + 'static
    {
        let taskid = crate::TASK_ID.next();
        let task: Box::<dyn Task<R=()> + Send + 'static> = Box::new(task);
        let mut lock = self.0.0.lock().unwrap();
        lock.insert(taskid, task).map_or(
            None, |_|Some(taskid))
    }
    fn remove(&self,id:usize)->Option<Box<dyn Task<R=()>+Send>> {
        let mut lock = self.0.0.lock().unwrap();
        lock.remove(&id)
    }
    fn update_c1<T:'static>(&self,id:usize,i:usize,v:T)->bool {
        let mut lock = self.0.0.lock().unwrap();
        let Some(task) = lock.get_mut(&id) else {
            return false;
        };
        let Some(param) = task.as_param_mut() else {
            return false;
        };
        param.set(i, &v)
    }
}

fn when_c1_comed<T:'static>(id:usize, i: usize, v:T, c1map: &C1map, q:Queue)->bool {
    if !c1map.update_c1(id, i, v) {
        return false;
    }
    let Some(task) = c1map.remove(id) else {
        return  false;
    };
    q.add_boxtask(task);
    true
}