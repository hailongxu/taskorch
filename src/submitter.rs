use crate::{
    cond::{ArgIdx, CondAddr, Place, TaskId}, curry::CallOnce, log::{Level,LEVEL}, meta::{Fndecl, Identical, TupleAt, TupleCondAddr}, queue::{C1map, WhenTupleComed}, task::{
        taskid_next, PsOf, Task, TaskCurrier, TaskMap, TaskNeed
    }, Queue
};

use std::{any::{type_name, Any, TypeId}, fmt::Debug, marker::PhantomData};

#[derive(Debug)]
pub enum TaskSubmitError {
    /// when submit task, if the id has already existed in waitQueue.
    TaskIdAlreadyExists(TaskId),
}

pub struct TaskInf<Ps> {
    taskid: TaskId,
    _phantom: PhantomData<Ps>,
}

impl<Ps> TaskInf<Ps> {
    pub(crate) const fn new(taskid:TaskId)->Self {
        Self { taskid, _phantom:PhantomData }
    }
    pub const fn taskid(&self)->TaskId {
        self.taskid
    }
}

impl<Args> TaskInf<Args> {
    pub fn input_at<const I:u8>(&self)->CondAddr<Args::EleT>
    where Args:TupleAt<I> {
        CondAddr::from((self.taskid, Place::Input, ArgIdx::const_new::<I>()))
    }
}

impl<Args> Debug for TaskInf<Args> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"TaskInf{{{:?} input<{}>}}",self.taskid(),type_name::<Args>())
    }
}

type SummitResult<Args> = Result<TaskInf<Args>,TaskSubmitError>;

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
    /// ```rust
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
    /// ```
    /// 
    /// # argments
    /// * `TaskBuild` - generate from `.into_task()` 
    /// * `task` - The main body of the task to be executed. 
    /// * `map` - Mapping function for processing and forwarding the task result
    /// 
    /// # returns
    /// * `SummitResult` - TaskInf or TaskError
    /// 
    /// * if taskid has already existed, return Error
    /// * if the task has no params and you donot fill an explicit taskid, 
    /// * here, return Ok(TaskId::NONE)
    /// * or else return Ok(TaskId)
    /// 
    /// 
    // TODO next: Optimize postdo: if no taskmap and no tofn, set Option<postdo> to None
    // instead of always invoking it indiscriminately. (the present)
    #[allow(private_bounds)]
    pub fn submit<C,MapFn,MapR,ToFn>(&self,TaskNeed{task,map:TaskMap(mapfn),tofn,..}:TaskNeed<C,MapFn,MapFn::R,ToFn>)->SummitResult<C::InputPs>
        where
        TaskCurrier<C>: Task,
        C: CallOnce + Send + 'static,
        C::R: 'static + Debug,
        C: PsOf,

        MapFn: Fndecl<(C::R,),MapR> + Send + 'static,
        MapFn::Pt: From<(<C as CallOnce>::R,)>,
        MapFn::Pt: Identical<(<C as CallOnce>::R,)>,
        MapFn::R: TupleCondAddr + Clone,

        ToFn: Send + 'static,
        for<'a> ToFn: Fndecl<(&'a MapFn::R,),<MapFn::R as TupleCondAddr>::TCA>,
        for<'a> <ToFn as Fndecl<(&'a MapFn::R,), <MapFn::R as TupleCondAddr>::TCA>>::Pt: From<(&'a MapFn::R,)>,
        for<'d,'e> (
            &'d MapFn::R,
            &'e <MapFn::R as TupleCondAddr>::TCA,
            // &'b <ToFn as Fndecl<(&'a MapFn::R,), <MapFn::R as TupleCondAddr>::Cat>>::R,
        ): WhenTupleComed,
        // here if we use 'd to substitue the 'e, the error occurs. ???
        for<'a,'c> &'a <MapFn::R as TupleCondAddr>::TCA: From<&'a <ToFn as Fndecl<(&'c MapFn::R,), <MapFn::R as TupleCondAddr>::TCA>>::R>,
        // if subsitue the 2nd 'a with 'b, will lead to error???
        for<'a,'c> &'a <MapFn::R as TupleCondAddr>::TCA: Identical<&'a <ToFn as Fndecl<(&'c MapFn::R,), <MapFn::R as TupleCondAddr>::TCA>>::R>,
    {
        let mk_postdo = |id:TaskId| {
            let c1map = self.c1map.clone();
            let c1queue = (self.qid,self.queue.clone());
            let postdo = move |r: Box<dyn Any>| {
                let r_from = &id;
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
                let rtuple = mapfn.call((r,).into());
                let rcondaddr = tofn.call(ToFn::Pt::from((&rtuple,)));
                (&rtuple, (&rcondaddr).into()).foreach(r_from, c1map, c1queue);

                // if the 'd and 'e is replaced by 'd, here will occer error.
                // because, the lifecycle of &rtuple and &rcondaddr are equal from func signatures.
                // but actually 
                // submitter.rs(110, 13): `rcondaddr` dropped here while still borrowed
                // drop(rcondaddr);
                // drop(rtuple); 
            };
            postdo
        };

        //
        // postdo maybe added another param of taskid indicating where the value comes from.
        // let postdo = Box::new(postdo);

        // without parameter
        if 0 == task.currier.count() {
            if LEVEL >= Level::Warn {
                if let TaskId(Some(_id)) = task.id {
                    warn!("Ignore the taskid {_id:?}: no conditions found for this task.");
                }
            }
            // if id is set we will check whether it is conflicted in map queue.
            // if let Some(id) = task.id {
                if self.c1map.check(task.id).is_some() {
                    error!("task#{:?} has existed in queue!!",task.id);
                    return Err(TaskSubmitError::TaskIdAlreadyExists(task.id))
                }
            // }

            let taskid = task.id;
            let task = Box::new(task);
            let postdo = Box::new(mk_postdo(taskid));
            self.queue.add_boxtask(task,postdo);
            debug!("task#{:?} added into Q#{}", taskid, self.qid);
            Ok(TaskInf::new(taskid))
        } else { // with parameters
            let mut task = task;
            if task.id.0.is_none() { task.id = taskid_next(); } // @A, ensure, the task.id is nonzero.
            let task = task;
            // task.id must be some
            let TaskId(Some(taskid)) = task.id else {
                unreachable!("task id has feeded in nonzero @A");
            };
            let postdo = Box::new(mk_postdo(task.id));
            let id = self.c1map.try_insert(task, postdo, taskid);
            if id.is_some() {
                debug_assert_eq!(Some(taskid),id);
                debug!("cond-task#{taskid:?} added into waitQueue");
                Ok(TaskInf::new(TaskId(id)))
            } else {
                error!("cond-task#{taskid:?} is duplicated and can not be added into waitQueue!");
                Err(TaskSubmitError::TaskIdAlreadyExists(TaskId(Some(taskid))))
            }
        }
        // Ok(TaskId::NONE)
    }

    #[deprecated(
        since="0.3.0",
        note = "Use `submit()` instead for strict type check. \
               `old_submit()` will be removed in next release."
    )]
    #[allow(private_bounds)]
    pub fn old_submit<C,MapFn,MapR,ToFn>(&self,taskneed:TaskNeed<C,MapFn,MapFn::R,ToFn>)->TaskId
        where
        TaskCurrier<C>: Task,
        C: CallOnce + Send + 'static,
        C::R: 'static + Debug,
        C: PsOf,

        MapFn: Fndecl<(C::R,),MapR> + Send + 'static,
        MapFn::Pt: From<(<C as CallOnce>::R,)>,
        MapFn::Pt: Identical<(<C as CallOnce>::R,)>,
        MapFn::R: TupleCondAddr + Clone,

        ToFn: Send + 'static,
        for<'a> ToFn: Fndecl<(&'a MapFn::R,),<MapFn::R as TupleCondAddr>::TCA>,
        for<'a> <ToFn as Fndecl<(&'a MapFn::R,), <MapFn::R as TupleCondAddr>::TCA>>::Pt: From<(&'a MapFn::R,)>,
        for<'d,'e> (
            &'d MapFn::R,
            &'e <MapFn::R as TupleCondAddr>::TCA,
            // &'b <ToFn as Fndecl<(&'a MapFn::R,), <MapFn::R as TupleCondAddr>::Cat>>::R,
        ): WhenTupleComed,
        // here up if we use 'd to substitue the 'e, the error occurs. ???
        for<'a,'c> &'a <MapFn::R as TupleCondAddr>::TCA:
            // if subsitue the 2nd 'a with 'b, will lead to error???
            From<&'a <ToFn as Fndecl<(&'c MapFn::R,), <MapFn::R as TupleCondAddr>::TCA>>::R> +
            Identical<&'a <ToFn as Fndecl<(&'c MapFn::R,), <MapFn::R as TupleCondAddr>::TCA>>::R>,
    {
        self.submit(taskneed).unwrap().taskid()
    }
}

#[cfg(test)]
impl TaskSubmitter {
    fn test_new() -> Self {
        Self {
            qid: 1,
            queue: Queue::new(),
            c1map: C1map::new(),
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

#[test]
fn test_submmit_construct() {
    use crate::task::TaskBuildNew;
    use crate::curry::Currier;
    use crate::task::PassthroughMapFn;
    use crate::task::OneToOne;
    let task: TaskNeed<Currier<_, (), ()>, PassthroughMapFn<()>, ((),), OneToOne<((),)>> = (||println!("task='free':  Hello, 1 2 3 .."),TaskId::from(1)).into_task();
    println!("task type={:?}", std::any::type_name_of_val(&task));
}

#[test]
fn test_submmit() {
    use crate::task::TaskBuildNew;
    let s = TaskSubmitter::test_new();
    let id1 = TaskId::new(1);
    let task = (|_:i32|(),id1).into_task();
    let task = s.submit(task);
    println!("i32:{}",type_name::<i32>());
    println!("debug of task: {task:?}");
    assert!(task.is_ok_and(|a|a.taskid()==id1));

    // repeat insert into task with same taskid, leading to an Err
    let task = (|_:i8|(),id1).into_task();
    let task = s.submit(task);
    println!("debug of task: {task:?}");
    assert!(
        task.is_err_and( |e| matches!( e, TaskSubmitError::TaskIdAlreadyExists(id) if {id==id1} ) )
    );
}

#[test]
fn test_taskinf() {
    let taskinf = TaskInf::<(i32,)>::new(TaskId::new(3));
}
