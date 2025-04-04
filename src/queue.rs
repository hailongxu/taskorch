use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    sync::{atomic::{AtomicBool, Ordering},Arc,
    Condvar, Mutex}, thread
};

use crate::{task::{ExitTask, NormalTask, Task, TaskKind, Which}, Jhandle};


type PostDo = dyn FnOnce(Box<dyn Any>) + Send;
// static  WHEN_NIL_COMED: Box<PostDo> = Box::new(|_|());
#[derive(Clone)]
pub struct Queue(Arc<(Mutex<VecDeque<(Box<dyn Task+Send>,Box<PostDo>)>>,Condvar)>);

impl Queue {
    pub fn new()->Self {
        Queue(Arc::new((Mutex::new(VecDeque::new()),Condvar::new())))
    }

    pub fn add_boxtask(&self,task:Box<dyn Task+Send>, postdo: Box<PostDo>) {
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back((task,postdo));
        if is_empty {
            self.0.1.notify_one();
        }
    }
    pub fn add<F,R>(&self,f:F,which:&Which,postdo: Box<PostDo>)
        where
        F:Fn()->R + Send+'static,
        R: Send+'static,
    {
        let task = NormalTask::from((f,which));
        let task = Box::new(task);
        self.add_boxtask(task,postdo);
    }

    pub fn addc1<F,P1,R>(&self,f:F,which:&Which,postdo: Box<PostDo>)
        where
        F:Fn(P1)->R + Send+'static,
        P1: Send+Clone+'static,
        R: Send+'static,
    {
        let task = NormalTask::from((f,which));
        let task = Box::new(task);
        self.add_boxtask(task,postdo);
    }

    pub fn add_exit(&self, f:impl Fn()+'static+Send) {
        let exit_task = ExitTask::from(f);
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back((Box::new(exit_task),Box::new(|_|when_nil_comed())));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn pop(&self)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
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
            if let Some((mut task,postdo)) = m.pop_front() {
                drop(m);
                let r = task.call_mut();
                if let Some(r) = r {
                    postdo(r);
                }
                if let TaskKind::Exit = task.kind() {
                    break;
                }
            }
            else {
                let _unused = queue.1.wait(m);
            }
        }
    });
    Jhandle(handle,quit_flag)
}


#[derive(Clone)]
pub(crate) struct C1map(Arc<(Mutex<HashMap<usize,(Box<dyn Task+Send>,Box<PostDo>)>>,Condvar)>);

impl C1map {
    pub(crate) fn new()->Self {
        Self(
            Arc::new((Mutex::new(HashMap::new()),Condvar::new()))
        )
    }
    pub(crate) fn insert<T>(&self,task: T,postdo:Box<PostDo>)->Option<usize>
    where T: Task + Send + 'static
    {
        let taskid = crate::TASK_ID.next();
        let task: Box::<dyn Task + Send + 'static> = Box::new(task);
        let mut lock = self.0.0.lock().unwrap();
        lock.insert(taskid, (task,postdo));
        Some(taskid)
    }
    fn remove(&self,id:usize)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
        let mut lock = self.0.0.lock().unwrap();
        lock.remove(&id)
    }
    fn update_c1<T:'static>(&self,which:&Which,v:T)->bool {
        let mut lock = self.0.0.lock().unwrap();
        let Some((task,postdo)) = lock.get_mut(&which.id) else {
            return false;
        };
        let Some(param) = task.as_param_mut() else {
            return false;
        };
        param.set(which.i, &v)
    }
}

pub(crate) fn when_c1_comed<T:'static>(which:&Which, v:T, c1map: C1map, q:Queue)->bool {
    if which.is_none() {
        return false;
    }
    if !c1map.update_c1(which, v) {
        return false;
    }
    let Some((task,postdo)) = c1map.remove(which.id) else {
        return  false;
    };
    q.add_boxtask(task,postdo);
    true
}

pub(crate) fn when_nil_comed() {}