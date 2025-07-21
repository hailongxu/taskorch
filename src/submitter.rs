use crate::{
    curry::CallOnce,
    meta::{Fndecl, Identical},
    queue::{when_ci_comed, C1map, WhenTupleComed},
    task::{Task, TaskMap},
    Queue,
    TaskCurrier
};

use std::{any::{Any, TypeId}, fmt::Debug};

/// Handles task submission to a specific queue
#[derive(Clone)]
pub struct TaskSubmitter {
    #[allow(dead_code)]
    pub(crate) qid: usize, // just use in log
    pub(crate) queue: Queue,
    pub(crate) c1map: C1map,
}

impl TaskSubmitter {
    /// Enqueues a new task for future scheduling
    ///
    /// # argments
    /// * `task` - The task to be added, wrapped in a `TaskCurrier`.
    /// * `taskid` - An optional identifier for the task, used for tracking.
    ///
    /// # returns
    /// * `usize` - The ID of the task
    #[allow(private_bounds)]
    pub fn submit<C,MapFn,MapR>(&self,(task,map):(TaskCurrier<C>,TaskMap<MapFn,MapR>))->usize
        where
        TaskCurrier<C>: Task,
        C: CallOnce + Send + 'static,
        C::R: 'static + Debug,
        MapFn: Fndecl<(C::R,),MapR> + Send + 'static,
        MapFn::Pt: From<(<C as CallOnce>::R,)>,
        MapFn::Pt: Identical<(<C as CallOnce>::R,)>,
        MapR: Send + 'static,
        MapFn::R: WhenTupleComed,
    {
        let c1map = self.c1map.clone();
        let c1queue = (self.qid,self.queue.clone());
        let postdo = move |r: Box<dyn Any>| {
            match map {
                TaskMap::None => return,
                // to single anchor
                TaskMap::To(to) => {
                    let _actual_type = r.type_id();
                    let Ok(r) = r.downcast::<C::R>() else {
                        let _expected_type = TypeId::of::<C::R>();
                        let _expected_type_name = std::any::type_name::<C::R>();
                        error!(
                            "to {to:?}.\ndowncast failed: expected {}, got {:?}",
                            _expected_type_name, _actual_type
                        );
                        panic!("failed to conver to R type");
                        // return;
                    };
                    let r: &C::R = &*r;
                    when_ci_comed(&to, r, c1map, c1queue);
                },
                // to multi-anchor
                TaskMap::ToMany(mapfn, _) => {
                    let _actual_type = r.type_id();
                    let Ok(r) = r.downcast::<C::R>() else {
                        let _expected_type = TypeId::of::<C::R>();
                        let _expected_type_name = std::any::type_name::<C::R>();
                        error!(
                            "to Many Anchors.\ndowncast failed: expected {}, got {:?}",
                            _expected_type_name, _actual_type
                        );
                        panic!("failed to conver to R type");
                        // return;
                    };
                    let r: C::R = *r;
                    // dispatch to multi-target
                    let rtuple = mapfn.call((r,).into());
                    rtuple.foreach(c1map, c1queue);
                }
            }
        };

        let postdo = Box::new(postdo);

        if 0 == task.currier.count() {
            let task = Box::new(task);
            self.queue.add_boxtask(task,postdo);
            debug!("task(#{}) added into Qid(#{})", usize::MAX, self.qid);
            usize::MAX
        } else {
            let taskid = task.id;
            let id = self.c1map.insert(task, postdo, taskid).unwrap();
            debug!("task(#{id}) with cond added into waitQueue");
            id
        }
    }
}

#[test]
fn test_conv() {
    use std::any::Any;
    let a = 3i32;
    let a: &dyn Any = &a;
    let b = a.downcast_ref::<i32>();
    assert!(b.is_some());
    let b = a.downcast_ref::<i8>();
    assert!(b.is_none());
    let b = a.downcast_ref::<i64>();
    assert!(b.is_none());
}
