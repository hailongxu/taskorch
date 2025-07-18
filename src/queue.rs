use std::{
    any::{type_name, Any},
    collections::{HashMap, VecDeque},
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex
    },
    thread
};

use crate::{task::{Task, Kind, Anchor}, Jhandle};
use crate::task::taskid_next;


type PostDo = dyn FnOnce(Box<dyn Any>) + Send;
// static  WHEN_NIL_COMED: Box<PostDo> = Box::new(|_|());

/// A queue holding tasks awaiting scheduling by threads
#[derive(Clone)]
pub struct Queue(Arc<(Mutex<VecDeque<(Box<dyn Task+Send>,Box<PostDo>)>>,Condvar)>);

impl Queue {
    pub fn new()->Self {
        Queue(Arc::new((Mutex::new(VecDeque::new()),Condvar::new())))
    }

    pub(crate) fn add_boxtask(&self,task:Box<dyn Task+Send>, postdo: Box<PostDo>) {
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back((task,postdo));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    #[allow(dead_code)]
    pub(crate) fn pop(&self)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
        self
            .0
            .0
            .lock()
            .unwrap()
            .pop_front()
    }
    
    #[allow(dead_code)]
    fn clear(&self) {
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
        warn!("starts ok.");
        loop {
            if quit.load(Ordering::Relaxed) {
                warn!("Quit flag detected and prepare to exit.");
                break;
            }
            
            let mut m = queue.0.lock().unwrap();
            if let Some((task,postdo)) = m.pop_front() {
                drop(m);
                let kind = task.kind();
                let r = task.run();
                if let Some(r) = r {
                    postdo(r);
                }
                if let Kind::Exit = kind {
                    warn!("received an exit message and prepare to exit.");
                    break;
                }
            } else {
                let _unused = queue.1.wait(m);
            }
        }
        warn!("exits ok.");
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
    pub(crate) fn insert<T>(&self,task: T,postdo:Box<PostDo>,taskid:Option<usize>)->Option<usize>
    where T: Task + Send + 'static
    {
        let mut taskid = taskid;
        let taskid = *taskid.get_or_insert_with(||taskid_next());
        let task: Box::<dyn Task + Send + 'static> = Box::new(task);
        let mut lock = self.0.0.lock().unwrap();
        lock.insert(taskid, (task,postdo));
        Some(taskid)
    }
    fn remove(&self,id:usize)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
        let mut lock = self.0.0.lock().unwrap();
        lock.remove(&id)
    }

    fn update_ci<T:'static+Debug>(&self,anchor:&Anchor,v:&T)->Option<bool> {
        let mut lock = self.0.0.lock().unwrap();
        let Some((task,_postdo)) = lock.get_mut(&anchor.id()) else {
            error!("task(#{}) was not found, the cond(#{}) could not be updated", anchor.id(), anchor.i());
            return None;
        };
        let Some(param) = task.as_param_mut() else {
            error!("task(#{}) failed to acquire cond(#{}), update skipped.", anchor.id(), anchor.i());
            return None;
        };
        if !param.set(anchor.i(), v) {
            let _taskid = anchor.id();
            let _i = anchor.i();
            let _this_type_name = param.typename(_i);
            let _data_type_name  = type_name::<T>();
            error!("task(#{_taskid}).cond#{_i} has type <{_this_type_name}> not identical to <{_data_type_name}>, \
                    cannot be updated with {{{v:?}}}.");
            return None;
        }
        Some(param.is_full())
    }
}

// qid just used for log
#[allow(unused_variables)]
pub(crate) fn when_ci_comed<T:'static+Debug>(to:&Anchor, v:&T, c1map:C1map, (qid,q):(usize,Queue))->bool {
    let Some(true) = c1map.update_ci(to, v) else {
        // the log has been processed in update_ci
        return false;
    };
    trace!("cond-task(#{}) receives cond(#{}) {{{v:?}}}", to.id(),to.i());
    let Some((task,postdo)) = c1map.remove(to.id()) else {
        error!("cond task(#{}) does not find.",to.id());
        return  false;
    };
    debug!("cond-task(#{}) has all been satified and schedued to Q(#{qid})", to.id());
    q.add_boxtask(task,postdo);
    true
}

#[allow(dead_code)]
pub(crate) fn when_nil_comed() {}



pub(crate) trait WhenTupleComed {
    fn foreach(&self, c1map:C1map, q:(usize,Queue));
}
impl<T:'static+Debug> WhenTupleComed for ((T,Anchor),) {
    fn foreach(&self, c1map:C1map, q:(usize,Queue)) {
        when_ci_comed(&self.0.1, &self.0.0, c1map, q);
    }
}

macro_rules! when_tuple_comed_impl {
    ($($i:tt $T:ident),+) => {
        impl< $($T:'static+Debug),+ > WhenTupleComed for ($(($T, Anchor)),+) {
            fn foreach(&self, c1map: C1map, q: (usize,Queue)) {
                $(
                    when_ci_comed(&self.$i.1, &self.$i.0, c1map.clone(), q.clone());
                )+
            }
        }
    };
}

#[cfg(false)]
macro_rules! when_tuple_comed_impl {
    ($(($t:ty, $n:tt)),+) => {
        // $t: ":"  ?????? error: expected one of `>` or `as`, found `:`
        // if $t 's type is ty. it is ok when $t is ident ???
        impl< $($t:'static+Debug),+ > WhenTupleComed for ($($t,Anchor),+) {
            fn foreach(&self, c1map:C1map, q:Queue) {
                $(
                    when_ci_comed(&self.$n.1, &self.$n.0, c1map, q);
                )+
            }
        }
    };
}

when_tuple_comed_impl!(0 T1, 1 T2);
when_tuple_comed_impl!(0 T1, 1 T2, 2 T3);
when_tuple_comed_impl!(0 T1, 1 T2, 2 T3, 3 T4);
when_tuple_comed_impl!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5);
when_tuple_comed_impl!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6);
when_tuple_comed_impl!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7);
when_tuple_comed_impl!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8);
