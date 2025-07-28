use crate::{
    curry::CallOnce,
    meta::{Fndecl, Identical},
    queue::{when_ci_comed, C1map, WhenTupleComed},
    task::{
        Task, TaskBuild, TaskCurrier, TaskMap, 
        TaskId, taskid_next
    },
    Queue,
    log::{Level,LEVEL},
};

use std::{any::{Any, TypeId}, fmt::Debug};

#[derive(Debug)]
pub enum TaskError {
    /// when submit task, if the id has already existed in waitQueue.
    TaskIdAlreadyExists(TaskId),
}
type SummitResult = Result<Option<TaskId>,TaskError>;

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
    /// # Examples:
    /// ```
    /// let task = (|a:i32|3,10).into_task(); // with explicit taskid=10
    /// let task = submitter.submit(task.into_task()); 
    /// assert_eq!(task,Ok(Some(TaskId::from(10))));
    /// 
    /// 
    /// let task = (||3).into_task();
    /// let task = submitter.submit(task.into_task()); 
    /// assert_eq!(task,Ok(None));
    /// 
    /// let task = (||3,1).into_task(); // with explicit taskid=1
    /// let task = submitter.submit(task.into_task()); 
    /// assert_eq!(task,Ok(Some(TaskId::from(1))));
    /// 
    /// // error, because 10 is used above.
    /// let task = (||3,10).into_task(); // with explicit taskid=10
    /// let task = submitter.submit(task.into_task()); 
    /// assert_eq!(task,Error(err_msg)));
    /// 
    /// ```
    /// # argments
    /// * `TaskBuild` - generate from `.into_task()` 
    /// * `task` - The main body of the task to be executed. 
    /// * `map` - Mapping function for processing and forwarding the task result
    /// 
    /// # returns
    /// * `Result<Option<TaskId>,String>` - The ID of the task
    /// 
    /// * if taskid has already existed, return Error
    /// * if the task has no params and you donot fill an explicit taskid, 
    /// * here, return Ok(None)
    /// * or else return Ok(Some(TaskId))
    /// 
    #[allow(private_bounds)]
    pub fn submit<C,MapFn,MapR>(&self,TaskBuild(task,map):TaskBuild<C,MapFn,MapR>)->SummitResult
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
        let mk_postdo = |id:Option<TaskId>| {
            let c1map = self.c1map.clone();
            let c1queue = (self.qid,self.queue.clone());
            let postdo = move |r: Box<dyn Any>| {
                let r_id = id;
                match map {
                    TaskMap::None => return,
                    // to single condaddr
                    TaskMap::To(to) => {
                        let _actual_type = r.type_id();
                        let Ok(r) = r.downcast::<C::R>() else {
                            let _expected_type = TypeId::of::<C::R>();
                            let _expected_type_name = std::any::type_name::<C::R>();
                            error!(
                                "task return value downcast failed: expected {}, got {:?}",
                                _expected_type_name, _actual_type
                            );
                            panic!("failed to conver to R type");
                            // return;
                        };
                        let r: &C::R = &*r;
                        when_ci_comed(&to, (r_id,r), c1map, c1queue);
                    },
                    // to multi-condaddr
                    TaskMap::ToMany(mapfn, _) => {
                        let _actual_type = r.type_id();
                        let Ok(r) = r.downcast::<C::R>() else {
                            let _expected_type = TypeId::of::<C::R>();
                            let _expected_type_name = std::any::type_name::<C::R>();
                            error!(
                                "task return value downcast failed: expected {}, got {:?}",
                                _expected_type_name, _actual_type
                            );
                            panic!("failed to conver to R type");
                            // return;
                        };
                        let r: C::R = *r;
                        // dispatch to multi-target
                        let rtuple = mapfn.call((r,).into());
                        rtuple.foreach(r_id, c1map, c1queue);
                    }
                }
            };
            postdo
        };

        //
        // postdo maybe added another param of taskid indicating where the value comes from.
        // let postdo = Box::new(postdo);

        // without parameter
        if 0 == task.currier.count() {
            if LEVEL >= Level::Warn {
                if let Some(_id) = task.id {
                    warn!("Ignore the taskid {_id:?}: no conditions found for this task.");
                }
            }
            // if id is set we will check whether it is conflicted in map queue.
            if let Some(id) = task.id {
                if self.c1map.check(id).is_some() {
                    error!("task#{:?} has existed in queue!!",task.id);
                    return Err(TaskError::TaskIdAlreadyExists(id))
                }
            }

            let taskid = task.id;
            let task = Box::new(task);
            let postdo = Box::new(mk_postdo(taskid));
            self.queue.add_boxtask(task,postdo);
            debug!("task#{:?} added into Q#{}", crate::task::TaskIdOption(taskid), self.qid);
            Ok(taskid)
        } else { // with parameters
            let mut task = task;
            if task.id.is_none() { task.id = Some(taskid_next()); }
            let task = task;
            let taskid = task.id.unwrap(); // task.id must be some
            let postdo = Box::new(mk_postdo(task.id));
            let id = self.c1map.try_insert(task, postdo, taskid);
            if id.is_some() {
                debug_assert_eq!(Some(taskid),id);
                debug!("cond-task#{taskid:?} added into waitQueue");
                Ok(id)
            } else {
                error!("cond-task#{taskid:?} is duplicated and can not be added into waitQueue!");
                Err(TaskError::TaskIdAlreadyExists(taskid))
            }
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
