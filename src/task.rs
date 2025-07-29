//! ## task module
//! 
//! Core scheduling concepts:
//! 
//! <1> CondAddr: Logical address to locate a condition (not a memory address)
//! 
//! <2> TaskId: Unique identifier for a task
//! 
//! <3> Pi: Zero-based index of a task parameter (also used as condition i
//! 

use std::{
    any::Any,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
    ops::{Deref,DerefMut},
    num::NonZeroUsize,
};

use crate::{curry::{CallOnce, CallParam, Currier}, meta::TupleOpt};
use crate::meta::Fndecl;


/// Defines the behavior type for tasks.
#[derive(Clone,Copy)]
pub enum Kind {
    /// Standard task execution.
    /// The thread continues running after task completion.
    Normal,
    /// Exit task execution.
    /// The current thread will exit automatically after the task completes.
    Exit,
}

static TASKID:TaskIdGen = TaskIdGen::new();

struct TaskIdGen {
    nexter: AtomicUsize
}
impl TaskIdGen {
    const fn new()->Self {
        Self {
            nexter: AtomicUsize::new(1)
        }
    }
    fn next(&self)->TaskId {
        TaskId::from(
            match self.nexter.fetch_add(1, Ordering::Relaxed) {
                0 => self.nexter.fetch_add(1, Ordering::Relaxed),
                id => id,
            }
        )
    }
}

/// Generate a task ID
/// * returns
/// * type `TaskId`
pub fn taskid_next()->TaskId {
    TASKID.next()
}


/// TaskId
/// the unique id of a pool instance system
/// 
/// TaskId supports both zero and non-zero values.
/// 
/// - Zero values are reserved for internal use (unconditional tasks)
/// - Callers cannot explicitly create zero IDs (attempts will log warnings)
/// - Non-zero IDs are used for normal conditional tasks
/// 
/// # Example:
/// ```
/// let taskid = TaskId::from(3);
/// let taskid = 3.into();
/// ```
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct TaskId(pub(crate) Option<NonZeroUsize>);

impl TaskId {
    const NONE : Self = Self(None);

    #[inline]
    pub fn new(id:usize)->Self {
        if crate::log::LEVEL as usize >= crate::log::Level::Warn as usize && id == 0 {
            warn!("TaskId::new() the input id is zero, is not avaiable!");
        }
        Self(NonZeroUsize::new(id))
    }
    #[inline]
    pub const fn as_usize(&self)->usize {
        match self.0 {
            Some(v) => v.get(),
            None => 0,
        }
    }
}

impl From<usize> for TaskId {
    fn from(id: usize) -> Self {
        Self::new(id)
    }
}

impl std::fmt::Debug for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(taskid) = self.0 {
            f.write_fmt(format_args!("TaskId({})",taskid.get()))
            // f.debug_tuple("TaskId").field(&v).finish()
        } else {
            f.write_fmt(format_args!("TaskId(None)"))
            // f.debug_tuple("TaskId").field(&v).finish()
        }
    }
}

// impl Deref for TaskId {
//     type Target = usize;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl DerefMut for TaskId {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// /// just for debug message
// #[doc(hidden)]
// #[repr(transparent)]
// pub(crate) struct TaskIdOption(pub(crate) Option<TaskId>);

// impl std::fmt::Debug for TaskIdOption {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if let Some(taskid) = self.0 {
//             f.write_fmt(format_args!("TaskId({})",taskid.as_usize()))
//             // f.debug_tuple("TaskId").field(&v).finish()
//         } else {
//             f.write_fmt(format_args!("TaskId(None)"))
//             // f.debug_tuple("TaskId").field(&v).finish()
//         }
//     }
// }

#[test]
fn test_tid() {
    let tid = TaskId::from(3);
    let tid: TaskId = 3.into();
    // let tid = TaskIdOption(Some(tid));
    // println!("{tid:?}");
    // let tid = TaskIdOption(None);
    // println!("{tid:?}");
}

/// A 0-based index of representing the position of a parameter in fun or closure signature.
/// 
/// # Examples:
/// ```rust
/// fn ff(a:i32,b:i16,c:char) {}
/// 
/// let pi = Pi::P0; // the index postion of 1st `a:i32` (index 0)
/// let pi = Pi::P1; // the index postion of 2nd `b:i16` (index 1)
/// let pi = Pi::P2; // the index postion of 3rd `c:char` (index 2)
/// let pi = Pi::from(2); // equivalent Pi::P2
/// let pi = 2.into(); // equivalent Pi::P2
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct Pi(pub(crate) u8);
impl Pi {
    pub const PI0:Pi = Pi(0);
    pub const PI1:Pi = Pi(1);
    pub const PI2:Pi = Pi(2);
    pub const PI3:Pi = Pi(3);
    pub const PI4:Pi = Pi(4);
    pub const PI5:Pi = Pi(5);
    pub const PI6:Pi = Pi(6);
    pub const PI7:Pi = Pi(7);
    pub const PI8:Pi = Pi(8);
}
impl Pi {
    const fn i(&self)->u8 {
        self.0
    }
}
impl From<u8> for Pi {
    #[inline]
    fn from(pi: u8) -> Self {
        Self(pi)
    }
}
impl From<Pi> for u8 {
    #[inline]
    fn from(pi: Pi) -> Self {
        pi.0
    }
}

/// Cond Addr
/// Represents the position where a condition occurs — specifically, the position of a parameter.
///
/// This is determined by a combination of the task ID and zero-based condition index,
/// which together uniquely identify where the parameter is located in the system.
pub struct CondAddr(TaskId,Pi);

impl CondAddr {
    #[inline]
    pub const fn taskid(&self)->TaskId {
        self.0
    }
    #[inline]
    pub const fn pi(&self)->Pi {
        self.1
    }
    #[inline]
    pub fn set(&mut self, id:TaskId, i:Pi) {
        self.0 = id;
        self.1 = i;
    }
}

impl From<(TaskId,Pi)> for CondAddr {
    fn from((tid,pi): (TaskId,Pi)) -> Self {
        Self(tid, pi)
    }
}


pub(crate) trait Task
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>>;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
    fn kind(&self)->Kind;
    fn id(&self)->TaskId;
}


/// The carrier of the task, used to create and invoke its functionality.
pub(crate) struct TaskCurrier<Currier> {
    pub(crate) currier: Currier,
    pub(crate) id: TaskId,
    pub(crate) kind: Kind,
}

pub(crate) enum TaskMap<MapFn,R> {
    None,
    To(CondAddr),
    ToMany(MapFn,PhantomData<R>),
}

impl<T> Task for TaskCurrier<T>
    where
    T: CallOnce,
    T::R: 'static,
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>> {
        let r = self.currier.call_once();
        if std::mem::size_of::<T::R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        self.currier.as_param_mut()
    }
    fn kind(&self)->Kind {
        self.kind
    }
    fn id(&self)->TaskId {
        self.id
    }
}

pub struct TaskBuild<C,MapFn,MapR>(pub(crate) TaskCurrier<C>,pub(crate) TaskMap<MapFn,MapR>);

impl<C,MapFn,MapR> TaskBuild<C,MapFn,MapR> {
    /// get task id from task, only if the task has conds.
    pub fn id(&self)->TaskId {
        self.0.id
    }
}

// This is done to prevent exposing `curry` to external users, thereby avoiding unnecessary complexity in the documentation.
// for the `to()` use the R of CallOnce:R, but it's just visibility inside crate.
pub trait RofCurrier {
    type Ret;
}
// here, we predefinetely know the R is excitley the type of F::CallOnce::R
impl<F,C:TupleOpt,R> RofCurrier for Currier<F,C,R> {
    type Ret = R;
}

impl<Currier:CallOnce+RofCurrier,R1> TaskBuild<Currier, NullMapFn<R1>,()>
{
    /// Configures the target condaddr to `(taskid, condid)`.
    /// # Arguments:
    /// * `taskid` - target task identifier
    /// * `i` - cond #index (0-based)
    // pub fn old_to(self, taskid:usize, i:usize) -> TaskBuild<Currier, NullMapFn<Currier::Ret>,()> {
    pub fn to(self, ca:CondAddr) -> TaskBuild<Currier, NullMapFn<Currier::Ret>,()> {
        TaskBuild (
            TaskCurrier {
                currier: self.0.currier,
                id: self.0.id,
                kind: self.0.kind,
            },
            TaskMap::To(ca)
        )
    }

    #[deprecated(
        since="0.3.0",
        note = "Use `to()` instead for strict type check. \
               `old_to()` will be removed in next release."
    )]
    pub fn old_to(self, to: usize, pi: usize) -> TaskBuild<Currier, NullMapFn<Currier::Ret>,()> {
        warn!("Use .to() instead, the .old_to() will be removed in next version.");
        debug_assert!(pi <= u8::MAX as usize);
        if pi > u8::MAX as usize {
            error!("The index of cond#{pi} is too large, shoul be <= {}.",u8::MAX);
        }
        self.to(CondAddr(TaskId::from(to), Pi::from(pi as u8)))
    }
}

impl<Currier:CallOnce,MapFn1,R1> TaskBuild<Currier, MapFn1,R1>
{
    /// Distribute the result of bady task to multi condaddrs.
    /// 
    /// # Example:
    /// 
    /// ```rust
    /// // the 1st task#1 receives cond i16 from task
    /// let task1 = (|_:i16|{},1).into_task(); 
    /// // the 2nd task#2 receives cond &'static str from task
    /// let task2 = (|_:&'static str|{},2).into_task();
    /// // create task pass the its value to task1 and task2
    /// let task  = (||3i32).into_task(); // the task main body with return type i32
    /// 
    /// // distribute the result to task1 and task2
    /// 
    /// // the type of input must be identical to return type of task body.
    /// task.fan_tuple_with(|_:i32|( 
    ///     (8i16,    CondAddr(1,0)), // the 1st branch output
    ///     ("apple", CondAddr(2,0)), // the 2nd branch output
    ///     ));
    /// ```
    /// 
    /// # Arguments:
    /// * mapfn: `FnOnce(Ret)->R`
    /// * `Ret` - the result type of task body 
    /// * `R` - is final output, must be two layer tuple. `((..),(..),..)`
    /// 
    /// # Returns:
    /// * format is a tuple, each elemtn is a tuple too
    /// * each element stands for an output
    /// ```rust
    /// (
    ///   value, /// the output of this branch  
    ///   CondAddr(taskid, cond#i) /// the target condaddr the value will be passed to.  
    /// )
    /// ```
    /// 
    /// * the whole structure of output is:
    /// ```rust
    ///  (  
    ///     (value1,CondAddr(task1,cond#i)), /// the 1st branch  
    ///     (value2,CondAddr(task2,cond#i)), /// the 2st branch  
    ///     ...  
    ///  )
    /// ```
    pub fn fan_tuple_with<MapFn,R>(self, mapfn:MapFn) -> TaskBuild<Currier, MapFn,R>
        where MapFn: Fndecl<(Currier::R,),R>
    {
        TaskBuild (
            TaskCurrier {
                currier: self.0.currier,
                id: self.0.id,
                kind: self.0.kind,
            },
            TaskMap::ToMany(mapfn, PhantomData),
        )
    }
}

/// TaskBuildOp provides target condaddr configuration.
#[deprecated(
    since="0.3.0",
    note = "Use `to()` directly, for this method has been integrated into the TaskBuild. \
           trait `TaskBuildOp` actually do nothing and will be removed in next release."
)]
pub trait TaskBuildOp<Currier,R> {}

/// A builder trait for constructing tasks with an optional task ID.
pub trait TaskBuildNew<C,F,R> {
    /// construct a task from a function or a closure or with an taskid.
    /// 
    /// # Example:
    /// ```rust
    /// 
    /// // no return
    /// let task = (||{}).into_task(); // just a function
    /// let task = (||{},1).into_task(); // function and an explicit taskid
    /// let task = (|_:i32|{}).into_task(); // task with one cond
    /// let task = (|_:i32|{},2).into_task(); // task with one cond and explicit taskid

    /// // with return
    /// let task = (||3).into_task(); // just a function
    /// let task = (||3,1).into_task(); // function and an explicit taskid
    /// let task = (|_:i32|3).into_task(); // task with one cond
    /// let task = (|_:i32|3,2).into_task(); // task with one cond and explicit taskid
    /// ```
    /// 
    /// # Arguments:
    /// * (fun,taskid:usize)
    /// * fun : a function or a closure with param count less equal 8
    /// * taskid: `usize`, you can also input the id explicitly
    /// 
    /// A `taskid` is required when the function has parameters, because other tasks
    /// need to know the location `CondAddr(taskid, cond#i)` to which they pass conditions.
    /// If the task has no parameters, the `taskid` is not required.
    /// However, if you omit it, the system will automatically generate a `taskid`.
    /// 
    /// # Returns
    /// 
    /// - TaskBuild: including the necessaary info of a task, 
    ///   just for preparetion of submition.
    fn into_task(self)->TaskBuild<C,F,R>;

    #[deprecated(
        since="0.2.0",
        note = "Use `into_task()` instead for clearer ownership semantic. \
               `task()` will be removed in next release."
    )]
    fn task(self)->TaskBuild<C,F,R> where Self:Sized {
        self.into_task()
    }

    /// construct a exit task
    /// # Note
    /// This is functionally identical to `into_task()`, with the additional behavior of thread exit gracefully
    /// after task completion.
    fn into_exit_task(self)->TaskBuild<C,F,R>;

    #[deprecated(
        since="0.2.0",
        note = "Use `into_exit_task()` instead for clearer ownership semantic. \
               `exit_task()` will be removed in next release."
    )]
    fn exit_task(self)->TaskBuild<C,F,R> where Self:Sized {
        self.into_exit_task()
    }
}
/// TaskBuildOp provides target condaddr configuration.
// pub trait TaskBuildOp<Currier,R> {
//     /// Configures the target condaddr to `(taskid, condid)`.
//     /// # Arguments:
//     /// * `taskid` - target task identifier
//     /// * `i` - cond #index (0-based)
//     fn to(self, taskid:usize,i:usize)->(TaskCurrier<Currier>,TaskMap<NullMapFn<R>,()>);
// }

// pub trait TaskBuildOpMany<Currier,MapFn,R>
//     where Currier: CallOnce,
// {
//     fn to_many(self, mapfn:MapFn)->(TaskCurrier<Currier>, TaskMap<MapFn,R>)
//         where MapFn: Fndecl<(Currier::R,),R>;
// }

// struct PassthroughMapFn;
// impl<P> Fndecl<(P,),P> for PassthroughMapFn {
//     type Pt=(P,);
//     type R=((P,);
//     fn call(self,_ps:Self::Pt)->Self::R {
//     }
// }

// impl<Currier> TaskBuildOp<Currier,Currier::R> for (TaskCurrier<Currier>, TaskMap<NullMapFn<Currier::R>,()>)
//     where
//     Currier: CallOnce,
// {
//     fn to(self, taskid:usize, i:usize) -> (TaskCurrier<Currier>, TaskMap<NullMapFn<Currier::R>,()>) {
//         (
//             TaskCurrier {
//                 currier: self.0.currier,
//                 id: self.0.id,
//                 kind: self.0.kind,
//             },
//             TaskMap::To(CondAddr(taskid, i))
//         )
//     }
// }

// impl<Currier,MapFn1,R1,MapFn,R> TaskBuildOpMany<Currier,MapFn,R> for (TaskCurrier<Currier>, TaskMap<MapFn1,R1>)
//     where
//     Currier: CallOnce,
// {
//     fn to_many(self, mapfn:MapFn) -> (TaskCurrier<Currier>, TaskMap<MapFn,R>)
//         where MapFn: Fndecl<(Currier::R,),R>
//     {
//         (
//             TaskCurrier {
//                 currier: self.0.currier,
//                 id: self.0.id,
//                 kind: self.0.kind,
//             },
//             TaskMap::ToMany(mapfn, PhantomData),
//         )
//     }
// }

#[test]
fn test_task_build_many() {
    let task = (||3).into_task();
    if true {
        task.fan_tuple_with(|_:i32| {
            ((3, CondAddr(TaskId::from(1), Pi(0))),)
        });
    } else {
        task.to(CondAddr(TaskId::from(3),Pi(0)));
    }
}

#[doc(hidden)]
pub struct NullMapFn<P> {
    phantom: PhantomData<P>
}
impl<P> Fndecl<(P,),()> for NullMapFn<P> {
    type Pt=(P,);
    type R=();
    fn call(self,_ps:Self::Pt)->Self::R {
    }
}
/// constructs a task without cond
impl<F:FnOnce()->R,R> TaskBuildNew<Currier<F,(),R>,NullMapFn<R>,()> for F {
    fn into_task(self) -> TaskBuild<Currier<F,(),R>,NullMapFn<R>,()> {
        TaskBuild (
            TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Normal,
            },
            TaskMap::None
        )
    }
    fn into_exit_task(self)->TaskBuild<Currier<F,(),R>,NullMapFn<R>,()> {
        TaskBuild (
            TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Exit,
            },
            TaskMap::None
        )
    }
}
impl<F:FnOnce()->R,R> TaskBuildNew<Currier<F,(),R>,NullMapFn<R>,()> for (F,TaskId) {
    fn into_task(self) -> TaskBuild<Currier<F,(),R>,NullMapFn<R>,()> {
        TaskBuild(
            TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Normal,
            },
            TaskMap::None
        )
    }
    fn into_exit_task(self) -> TaskBuild<Currier<F,(),R>,NullMapFn<R>,()> {
        TaskBuild (
            TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Exit,
            },
            TaskMap::None
        )
    }
}

impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<Currier<F,(P1,),R>,NullMapFn<R>,()> for F {
    fn into_task(self) -> TaskBuild<Currier<F,(P1,),R>,NullMapFn<R>,()> {
        TaskBuild(
            TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Normal,
            },
            TaskMap::None
        )
    }
    fn into_exit_task(self) -> TaskBuild<Currier<F,(P1,),R>,NullMapFn<R>,()> {
        TaskBuild (
            TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Exit,
            },
            TaskMap::None
        )
    }
}
impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<Currier<F,(P1,),R>,NullMapFn<R>,()> for (F,TaskId) {
    fn into_task(self) -> TaskBuild<Currier<F,(P1,),R>,NullMapFn<R>,()> {
        TaskBuild (
            TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Normal,
            },
            TaskMap::None
        )
    }
    fn into_exit_task(self) -> TaskBuild<Currier<F,(P1,),R>,NullMapFn<R>,()> {
        TaskBuild (
            TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Exit,
            },
            TaskMap::None
        )
    }
}

macro_rules! impl_task_build_new {
    ($($P:ident),+) => {
        impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<Currier<F, ($($P,)+), R>,NullMapFn<R>,()> for F {
            fn into_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, NullMapFn<R>,()> {
                TaskBuild (
                    TaskCurrier {
                        currier: Currier::from(self),
                        id: TaskId::NONE,
                        kind: Kind::Normal,
                    },
                    TaskMap::None
                )
            }
            
            fn into_exit_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, NullMapFn<R>,()> {
                TaskBuild (
                    TaskCurrier {
                        currier: Currier::from(self),
                        id: TaskId::NONE,
                        kind: Kind::Exit,
                    },
                    TaskMap::None
                )
            }
        }


        impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<Currier<F, ($($P,)+), R>, NullMapFn<R>,()> for (F, TaskId) {
            fn into_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, NullMapFn<R>,()> {
                TaskBuild (
                    TaskCurrier {
                        currier: Currier::from(self.0),
                        id: self.1,
                        kind: Kind::Normal,
                    },
                    TaskMap::None
                )
            }
            
            fn into_exit_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, NullMapFn<R>,()> {
                TaskBuild (
                    TaskCurrier {
                        currier: Currier::from(self.0),
                        id: self.1,
                        kind: Kind::Exit,
                    },
                    TaskMap::None
                )
            }
        }
    };
}

impl_task_build_new!(P1,P2);
impl_task_build_new!(P1,P2,P3);
impl_task_build_new!(P1,P2,P3,P4);
impl_task_build_new!(P1,P2,P3,P4,P5);
impl_task_build_new!(P1,P2,P3,P4,P5,P6);
impl_task_build_new!(P1,P2,P3,P4,P5,P6,P7);
impl_task_build_new!(P1,P2,P3,P4,P5,P6,P7,P8);


#[test]
fn test_task_new() {
    let f = ||();
    let t = f.into_exit_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let s = String::new();
    let f = ||{let _s=s;};

    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();
}

#[should_panic]
#[test]
fn test_task_new_panic() {
    let f = |_:i32,_:i32|{};
    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.0);
    // the param is not set, so panic
    t.run();
}

#[test]
fn test_task_postdo() {
    let mut v = 3;
    let f = ||{v=3;v};
    let v = Some(String::new());
    let postdo = |_:i32|{v.unwrap();};
    let _r1: &dyn FnMut()->i32 = &f;
    let r1: Box<dyn FnOnce(i32)> = Box::new(postdo);
    r1(3);
}
#[test]
fn test_task_run() {
    // one cond
    let c1 = (|_p:i32|println!("get c1")).into_task();
    let mut c1: Box<dyn Task> = Box::new(c1.0);
    c1.as_param_mut().map(|e|e.set(0, &5));
    c1.run();

    // 8 cond
    let tp1 = 1;
    let tp2 = "2nd static str";
    let tp3 = "3rd String".to_string();
    let tp4 = vec![41,42,43];
    let tp5 = 5;
    let tp6 = 6;
    let tp7 = 7;
    let tp8 = 8;
    let tr8 = tp1+tp5+tp6+tp7+tp8;
    let c8 = (
        |p1:i32,
         p2:&'static str,
         p3:String,
         p4:Vec<i32>,
         p5:i32,
         p6:i32,
         p7:i32,
         p8:i32|{
        assert_eq!(p1,tp1);
        assert_eq!(p2,tp2);
        assert_eq!(p3,tp3);
        assert_eq!(p4,tp4);
        assert_eq!(p5,tp5);
        assert_eq!(p6,tp6);
        assert_eq!(p7,tp7);
        assert_eq!(p8,tp8);
        println!("recevied cond: {p1},{p2},{p3},{p4:?},{p5},{p6},{p7},{p8},");
        p1+p5+p6+p7+p8
    }).into_task();
    let mut c8: Box<dyn Task> = Box::new(c8.0);
    c8.as_param_mut().map(
        |e|
        e.set(0, &tp1) && 
        e.set(1, &tp2) && 
        e.set(2, &tp3) && 
        e.set(3, &tp4) && 
        e.set(4, &tp5) && 
        e.set(5, &tp6) && 
        e.set(6, &tp7) && 
        e.set(7, &tp8)
    );
    let r = c8.run().unwrap();
    let r = r.downcast::<i32>().unwrap();
    assert_eq!(*r, tr8);
}
