use std::{
    any::{type_name, Any, TypeId}, collections::{HashMap, VecDeque}, fmt::Debug, num::NonZeroUsize, sync::{
        atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex
    }, thread
};

use crate::{task::{CondAddr, Kind, Task, TaskId}, Jhandle};

// enum InsertError {
//     /// task is must not be null
//     TaskIdIsNull,
// }

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
                debug!("task#{:?} is scheduled to run.",task.id());
                let kind = task.kind();
                let r = task.run();
                // if let Some(r) = r {
                    postdo(r);
                // }
                if let Kind::Exit = kind {
                    warn!("current thread received an exit message and prepare to exit.");
                    break;
                }
            } else {
                let _unused = queue.1.wait(m);
            }
        }
        info!("current thread exited normally.");
    });
    Jhandle(handle,quit_flag)
}

#[derive(Clone)]
pub(crate) struct C1map(Arc<(Mutex<HashMap<NonZeroUsize,(Box<dyn Task+Send>,Box<PostDo>)>>,Condvar)>);

impl C1map {
    pub(crate) fn new()->Self {
        Self(
            Arc::new((Mutex::new(HashMap::new()),Condvar::new()))
        )
    }
    pub(crate) fn check(&self, tid:TaskId)->Option<TaskId> {
        let TaskId(Some(ref taskid)) = tid else {
            return None;
        };

        let lock = self.0.0.lock().unwrap();
        if lock.contains_key(taskid) {
            Some(tid)
        } else {
            None
        }
    }
    pub(crate) fn try_insert<T>(&self,task: T,postdo:Box<PostDo>,taskid:NonZeroUsize)->Option<NonZeroUsize>
    where T: Task + Send + 'static
    {
        let task: Box::<dyn Task + Send + 'static> = Box::new(task);
        let mut lock = self.0.0.lock().unwrap();
        use std::collections::hash_map::Entry::{Occupied,Vacant};
        match lock.entry(taskid) {
            Occupied(_occupied_entry)
                => None,
            Vacant(vacant_entry)
                => {
                vacant_entry.insert((task,postdo));
                Some(taskid)
            },
        }
    }
    fn remove(&self,id:&NonZeroUsize)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
        let mut lock = self.0.0.lock().unwrap();
        lock.remove(id)
    }

    // Some(true): full
    // Some(false): not full
    // None: error
    fn update_ci<T:'static+Debug>(&self,target_ca:&CondAddr<T>,(v,v_from):(&T,&TaskId))->Option<bool> {
        let TaskId(Some(ref target_taskid)) = target_ca.taskid() else {
            error!("task#{:?} is ZERO, not avaiable!", target_ca.taskid());
            return None;
        };
        let mut lock = self.0.0.lock().unwrap();
        let Some((target_task,_target_postdo)) = lock.get_mut(target_taskid) else {
            error!("task#{:?} was not found, the cond#{:?} could not be updated", target_ca.taskid(), target_ca.pi());
            return None;
        };
        let Some(param) = target_task.as_param_mut() else {
            error!("task#{:?} failed to acquire cond#{:?}, update skipped.", target_ca.taskid(), target_ca.pi());
            return None;
        };
        if !param.set(target_ca.pi().0 as usize, v) {
            let _target_taskid = target_ca.taskid();
            let _target_i = target_ca.pi();
            let _target_type_name = param.typename(_target_i.0 as usize);
            let _data_type_name  = type_name::<T>();
            error!("target task#{_target_taskid:?}.cond#{_target_i:?} has type <{_target_type_name}> not identical to <{_data_type_name}>, \
                    cannot be updated with from task#{v_from:?}.{{{v:?}}}.");
            return None;
        }
        if cfg!(feature="log-trace") {
            trace!("target task{{{target_ca:?}}} received from task{{{v_from:?}}}={{{v:?}}}");
        } else {
            debug!("target task{{{target_ca:?}}} received from task{{{v_from:?}}}");
        }
        Some(param.is_full())
    }
}

// tid and qid just used for log
#[allow(unused_variables)]
pub(crate) fn when_ci_comed<T:'static+Debug>(target_ca:&CondAddr<T>, (v,v_from):(&T,&TaskId), c1map:C1map, (qid,q):(usize,Queue))->bool {
    let Some(true) = c1map.update_ci(target_ca,(v,v_from)) else {
        // the log has been processed in update_ci
        return false;
    };

    let TaskId(Some(ref target_taskid)) = target_ca.taskid() else {
        unreachable!("the taskid has checked in update_ci()!");
        return false;
    };
    let Some((target_task,postdo)) = c1map.remove(target_taskid) else {
        error!("cond task#{:?} does not find.",target_ca.taskid());
        return  false;
    };
    debug!("cond task#{:?} has all conditions been satified and scheduled to Q#{qid}", target_ca.taskid());
    q.add_boxtask(target_task,postdo);
    true
}

#[allow(dead_code)]
pub(crate) fn when_nil_comed() {}



pub(crate) trait WhenTupleComed {
    fn foreach(&self, id_from:&TaskId, c1map:C1map, q:(usize,Queue));
}

impl WhenTupleComed for () {
    fn foreach(&self, _id_from:&TaskId,_c1map:C1map, _q:(usize,Queue)) {
        // debug!("---type id:{:?} namne:{:?}", ().type_id(),type_name::<()>())
    }
}

impl<'a,'b, T:'static+Debug> WhenTupleComed for (&'a(T,),&'b(CondAddr<T>,)) {
// impl<T1:'static+Debug,T2:'static+Debug> WhenTupleComed for ((T1,),(T2,)) {
    fn foreach(&self, id_from:&TaskId, c1map:C1map, q:(usize,Queue)) {
        // debug!("---type id:{:?} name:{:?}", TypeId::of::<T1>(),type_name::<T1>());
        // debug!("---type id:{:?} name:{:?}", TypeId::of::<T2>(),type_name::<T2>());

        when_ci_comed(&self.1.0, (&self.0.0,id_from), c1map, q);
    }
}

macro_rules! when_tuple_comed_impl {
    ($($i:tt $T:ident),+) => {
        impl< $($T:'static+Debug),+ > WhenTupleComed for (&($($T),+), &($(CondAddr<$T>),+)) {
            fn foreach(&self, id_from:&TaskId, c1map: C1map, q: (usize,Queue)) {
                $(
                    // debug!("---type id:{:?} name:{:?}", TypeId::of::<$T>(),type_name::<$T>());
                    when_ci_comed(&self.1.$i, (&self.0.$i,id_from), c1map.clone(), q.clone());
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
        impl< $($t:'static+Debug),+ > WhenTupleComed for ($($t,CondAddr),+) {
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


#[test]
fn test_when_tuple_comed() {
    use crate::task::TaskId;
    use crate::queue::{C1map, Queue};
    use std::num::NonZeroUsize;

    let c1map = C1map::new();
    let q = Queue::new();
    let id_from = TaskId::new(1);
    
    let cond_addr1 = CondAddr::<i32>::const_new::<0>();
    let taskid1 = NonZeroUsize::new(2).unwrap();
    let cond_addr2 = CondAddr::<i32>::const_new::<1>();
    let taskid2 = NonZeroUsize::new(2).unwrap();

    ().foreach(&id_from, c1map.clone(), (0,q.clone()));
    (&(42,43), &(cond_addr1,cond_addr2)).foreach(&id_from, c1map.clone(), (0,q.clone()));
    // ((42,), (cond_addr,)).foreach(&id_from, c1map.clone(), (0,q.clone()));
}