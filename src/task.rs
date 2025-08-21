//! ## task module
//! 
//! Core scheduling concepts:
//! 
//! <1> CondAddr: Logical address to locate a condition (not a memory address)
//! Each Cond belongs to task, which is identified/found by taskid.
//! Each task has many Params, which is identified/found by Pi.
//! hence, the condaddr can be unique be located by taskid and paramter index.
//! Contruct a CondAddr via `from()`.
//! ## Exmaples:
//! ```rust
//! // cond addr is at (Task#1 and Task.Param#0) 
//! let ca = CondAddr::from((TaskId::new(1),Pi::PI0)); 
//! ```
//! 
//! <2> TaskId: Unique identifier for a task
//! 
//! <3> Pi: Zero-based index of a task parameter (also used as condition i)
//! 

use std::{
    any::Any,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
    num::NonZeroUsize,
};

use crate::{curry::{CallOnce, CallParam, Currier}, meta::{TupleAt, TupleCondAddr, TupleOpt}};
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
/// - except TaskId::None
pub fn taskid_next()->TaskId {
    TASKID.next()
}



/// TaskId
/// the unique id of a pool instance system
/// 
/// enforce `TaskId` zero/non-zero semantics
/// - Zero `TaskId` is now strictly internal (auto-assigned for unconditional tasks)
/// - Caller attempts to create zero IDs will log warnings
/// - Explicit non-zero IDs are required for all tasks (both conditional and unconditional). 
/// 
/// You can also convert the `TaskId` to `usize`, using `.as_usize()`.
/// 
/// # Example:
/// ```
/// let taskid = TaskId::from(3);
/// let taskid = 3.into();
/// let uid = taskid.as_usize();
/// ```
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct TaskId(pub(crate) Option<NonZeroUsize>);

impl TaskId {
    pub(crate) const NONE : Self = Self(None);

    /// Construct a TaskId from usize
    ///
    /// # Behavior
    /// - For non-zero IDs: Always succeeds
    /// - For zero ID:
    ///   - In debug mode: panics immediately
    ///   - In release mode: logs warning but returns `TaskId::NONE`
    /// 
    /// Examples:
    /// ```rust
    /// # use taskorch::task::TaskId; 
    /// let taskid = TaskId::new(1); // ok
    /// ```
    /// 
    /// Debug mode panic example (only compiles in debug):
    /// ```should_panic
    /// # use taskorch::task::TaskId;
    /// TaskId::new(0); // panics in debug
    /// ```
    ///
    /// Release mode behavior demonstration:
    /// ```no_run
    /// # use taskorch::task::TaskId;
    /// let _ = TaskId::new(0); // would log warning in release
    /// ```
    #[inline]
    pub fn new(id:usize)->Self {
        #[cfg(debug_assertions)]
        if id == 0 {
            panic!("TaskId cannot be zero");
        }
        #[cfg(not(debug_assertions))]
        if crate::log::LEVEL as usize >= crate::log::Level::Warn as usize && id == 0 {
            warn!("TaskId::new() the input id is zero, is not avaiable!");
        }
        Self(NonZeroUsize::new(id))
    }

    /// convert the TskId to usize
    #[inline]
    pub const fn as_usize(&self)->usize {
        match self.0 {
            Some(v) => v.get(),
            None => 0,
        }
    }
}

/// construct a `TaskId` from a `usize`
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
    let tid = TaskId::new(0);
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
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct Pi<T>(pub(crate) u8,PhantomData<T>);
impl<T> Pi<T> {
    pub const PI0:Pi<T> = Pi(0,PhantomData);
    pub const PI1:Pi<T> = Pi(1,PhantomData);
    pub const PI2:Pi<T> = Pi(2,PhantomData);
    pub const PI3:Pi<T> = Pi(3,PhantomData);
    pub const PI4:Pi<T> = Pi(4,PhantomData);
    pub const PI5:Pi<T> = Pi(5,PhantomData);
    pub const PI6:Pi<T> = Pi(6,PhantomData);
    pub const PI7:Pi<T> = Pi(7,PhantomData);
    pub const PI8:Pi<T> = Pi(8,PhantomData);
    const  PINONE:Pi<T> = Pi(u8::MAX,PhantomData);

    const fn const_new<const i:u8>() -> Self {
        Pi(i,PhantomData)
    }
}
impl<T> Pi<T> {
    const fn i(&self)->u8 {
        self.0
    }
}
impl<T> From<u8> for Pi<T> {
    #[inline]
    fn from(pi: u8) -> Self {
        Self(pi,PhantomData)
    }
}
impl<T> From<Pi<T>> for u8 {
    #[inline]
    fn from(pi: Pi<T>) -> Self {
        pi.0
    }
}

/// Cond Addr
/// Represents the position where a condition occurs — specifically, the position of a parameter.
///
/// This is determined by a combination of the task ID and zero-based condition index,
/// which together uniquely identify where the parameter is located in the system.
// #[derive(Clone, Copy)]
#[derive(Debug,PartialEq)]
pub struct CondAddr<T>(TaskId,Pi<T>);
    // where Pi<T>: Copy;
impl<T> CondAddr<T> {
    pub const NONE: Self = Self(TaskId::NONE,Pi::PINONE);
    pub(crate) const fn const_new<const i:u8>()->Self {
        Self(TaskId::NONE, Pi::const_new::<i>())
    }
}
impl<T> CondAddr<T> {
    #[inline]
    pub const fn taskid(&self)->TaskId {
        self.0
    }
    #[inline]
    pub const fn pi(&self)->&Pi<T> {
        &self.1
    }
    #[inline]
    pub fn set(&mut self, id:TaskId, i:Pi<T>) {
        self.0 = id;
        self.1 = i;
    }
    #[inline]
    pub fn set_taskid(&mut self, id:TaskId) {
        self.0 = id
    }
}

impl<T> Default for CondAddr<T> {
    fn default() -> Self {
        Self(TaskId::NONE, Pi::PINONE)
    }
}

impl<T> From<(TaskId,Pi<T>)> for CondAddr<T> {
    fn from((tid,pi): (TaskId,Pi<T>)) -> Self {
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

pub(crate) struct TaskMap<MapFn>(pub(crate) MapFn);

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

pub struct TaskBuild<C,MapFn,MapR,ToFn>
    where MapR: TupleCondAddr
{
    pub(crate) task: TaskCurrier<C>,
    pub(crate) map: TaskMap<MapFn>,
    pub(crate) tofn: ToFn,
    pub(crate) phantom: PhantomData<MapR>
}

impl<C,MapFn,MapR:TupleCondAddr,ToFn> TaskBuild<C,MapFn,MapR,ToFn> {
    /// get task id from task, only if the task has conds.
    pub fn id(&self)->TaskId {
        self.task.id
    }
}

// This is done to prevent exposing `curry` to external users, thereby avoiding unnecessary complexity in the documentation.
// for the `to()` use the R of CallOnce:R, but it's just visibility inside crate.
// #[doc(hidden)]
// pub trait RofCurrier {
//     type Ret;
// }
// // here, we predefinetely know the R is excitley the type of F::CallOnce::R
// impl<F,C:TupleOpt,R> RofCurrier for Currier<F,C,R> {
//     type Ret = R;
// }

// pub(crate) fn pass_through<T>(t:T)->(T,) {
//     (t,)
// }

#[test]
fn test_pass_through() {
    trait Function {}
    impl<F:FnOnce()> Function for F {}

    fn do_nothing() {}
    struct AA<F:Function>(F);
    impl<F:Function> AA<F> {
        // how does we construct a AA???
        #[cfg(false)]
        fn test()->Self {
            let a = AA(do_nothing);
            a
        }
        fn test2() {
            let a = AA(do_nothing);
            let AA(_f) = a;
            // error
            #[cfg(false)]
            let a = Self(do_nothing);
        }
    }
}

// impl<Currier:CallOnce+RofCurrier> TaskBuild<Currier, PassthroughMapFn<Currier::Ret>,()>
impl<F,TC,R,MapFn1,R1,ToFn1> TaskBuild<Currier<F,TC,R>, MapFn1,R1,ToFn1>
    where
    TC: TupleOpt,
    R1:TupleCondAddr,
{
    /// Configures the target condaddr to `(taskid, condid)`.
    /// attention:
    /// if the task has no result, the the to configuration is ignored.
    /// 
    /// # Arguments:
    /// * ca:`CondAddr` - the target cond place. target task identifier
    /// * `i` - cond #index (0-based)
    /// 
    // pub fn old_to(self, taskid:usize, i:usize) -> TaskBuild<Currier, PassthroughMapFn<Currier::Ret>,()> {
    // Tt: Target Type
    pub fn to<'a>(self, ca:CondAddr<R>)
        -> TaskBuild<
            Currier<F,TC,R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>,
            // impl Fndecl<(R,),(R,)>,
            // (R,),
            // impl Fndecl<(&'a (R,),),CondAddr<R>>
        >
    {
        let map  = TaskMap(PassthroughMapFn::<R>::NULL);
        let tofn = OneToOne::<(R,)>((ca,));
        // let tofn = move |_:&(R,)| ca;
        TaskBuild {
            task: self.task,
            map,
            tofn,
            phantom: PhantomData,
        }
    }

    #[deprecated(
        since="0.3.0",
        note = "Use `to()` instead for strict type check. \
               `old_to()` will be removed in next release."
    )]
    // pub fn old_to(self, to: usize, pi: usize) -> TaskBuild<Currier, PassthroughMapFn<Currier::R>,(Currier::R,)>
    pub fn old_to<'a>(self, to: usize, pi: usize) 
        -> TaskBuild<
            Currier<F,TC,R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>,
        >
    {
        warn!("Use .to() instead, the .old_to() will be removed in next version.");
        debug_assert!(pi <= u8::MAX as usize);
        if pi > u8::MAX as usize {
            error!("The index of cond#{pi} is too large, shoul be <= {}.",u8::MAX);
        }
        self.to(CondAddr(TaskId::from(to), Pi::from(pi as u8)))
    }
}

impl<F,TC,R,MapFn1,R1,ToFn1> TaskBuild<Currier<F,TC,R>, MapFn1,R1,ToFn1>
    where
    TC: TupleOpt,
    R1: TupleCondAddr,
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
    pub fn fan_tuple_with<'a,MapFn,MapR>(self, mapfn:MapFn)
        -> TaskBuild<
            Currier<F,TC,R>,
            MapFn,
            // impl Fndecl<(R,),MapR>,
            MapR,
            OneToOne::<MapR>,
        >
        where
        MapR: TupleCondAddr,
        MapR::Cat: Default,
        MapFn: Fndecl<(R,),MapR>,
    {
        TaskBuild {
            task: self.task,
            map: TaskMap(mapfn),
            tofn: OneToOne::<MapR>::ONETOONE,
            phantom: PhantomData,
        }
    }
}

impl<F,TC,R,MapFn1,R1> TaskBuild<Currier<F,TC,R>, MapFn1,R1,OneToOne<R1>>
    where
    TC: TupleOpt,
    R1: TupleCondAddr,
{
    pub fn all_to(mut self, cat: R1::Cat)->Self {
        self.tofn.0 = cat;
        self
    }
}

impl<F,TC,R,MapFn1,R1> TaskBuild<Currier<F,TC,R>, MapFn1,R1,OneToOne<R1>>
    where
    TC: TupleOpt,
    R1: TupleCondAddr,
{
    pub fn input_at<const I:u8>(&self)->CondAddr<TC::T>
        where TC: TupleAt<I>
    {
        CondAddr(self.id(), Pi::from(I))
    }
    pub fn output_at<const I:u8>(&self)->CondAddr<R1::T>
        where R1: TupleAt<I>
    {
        CondAddr(self.id(), Pi::from(I))
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
pub trait TaskBuildNew<C,F,R,T> {
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
    fn into_task(self)->TaskBuild<C,F,R,T> where R: TupleCondAddr;

    #[deprecated(
        since="0.2.0",
        note = "Use `into_task()` instead for clearer ownership semantic. \
               `task()` will be removed in next release."
    )]
    fn task(self)->TaskBuild<C,F,R,T> where Self:Sized, R: TupleCondAddr {
        self.into_task()
    }

    /// construct a exit task
    /// # Note
    /// This is functionally identical to `into_task()`, with the additional behavior of thread exit gracefully
    /// after task completion.
    fn into_exit_task(self)->TaskBuild<C,F,R,T> where R:TupleCondAddr;

    #[deprecated(
        since="0.2.0",
        note = "Use `into_exit_task()` instead for clearer ownership semantic. \
               `exit_task()` will be removed in next release."
    )]
    fn exit_task(self)->TaskBuild<C,F,R,T> where Self:Sized, R:TupleCondAddr {
        self.into_exit_task()
    }
}

/////////
// / TaskBuildOp provides target condaddr configuration.
// pub trait TaskBuildOp<Currier,R> {
//     /// Configures the target condaddr to `(taskid, condid)`.
//     /// # Arguments:
//     /// * `taskid` - target task identifier
//     /// * `i` - cond #index (0-based)
//     fn to(self, taskid:usize,i:usize)->(TaskCurrier<Currier>,TaskMap<PassthroughMapFn<R>,()>);
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

// impl<Currier> TaskBuildOp<Currier,Currier::R> for (TaskCurrier<Currier>, TaskMap<PassthroughMapFn<Currier::R>,()>)
//     where
//     Currier: CallOnce,
// {
//     fn to(self, taskid:usize, i:usize) -> (TaskCurrier<Currier>, TaskMap<PassthroughMapFn<Currier::R>,()>) {
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
// }//////

#[test]
fn test_task_build_many() {
    let task = (||3).into_task();
    if true {
        task.fan_tuple_with(|_:i32| (3,));
    } else {
        task.to(CondAddr(TaskId::from(3),Pi::<i32>::PI0));
    }

    let task = (||{}).into_task();
    match 0 {
        0 => {task.fan_tuple_with(|_:()| (3,) ); }
        1 => {task.to(CondAddr(TaskId::from(3),Pi::PI0)); }
        2 => {task.old_to(1,2); }
        _ => {}
    }
}

#[doc(hidden)]
pub struct PassthroughMapFn<P> {
    phantom: PhantomData<P>
}
impl<P> PassthroughMapFn<P> {
    const NULL:Self = Self {phantom:PhantomData};
}

impl<P> Fndecl<(P,),(P,)> for PassthroughMapFn<P> {
    type Pt=(P,);
    type R=(P,);
    fn call(self,ps:Self::Pt)->Self::R {
        ps
    }
}
pub struct OneToOne<Rtuple:TupleCondAddr>(Rtuple::Cat);
impl<P:TupleCondAddr> OneToOne<P>
{
    const ONETOONE:Self = Self(P::ONETOONE);
}
impl<'a,Rtuple:TupleCondAddr> Fndecl<(&'a Rtuple,),Rtuple::Cat> for OneToOne<Rtuple> {
    type Pt = (&'a Rtuple,);
    type R = Rtuple::Cat;
    fn call(self,_ps:Self::Pt)->Self::R {
        self.0
    }
}
/// constructs a task without cond
impl<F:FnOnce()->R,R> TaskBuildNew<Currier<F,(),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for F {
    fn into_task(self)
        -> TaskBuild<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Normal,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
    fn into_exit_task(self)
        -> TaskBuild<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Exit,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
}

impl<F:FnOnce()->R,R> TaskBuildNew<Currier<F,(),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for (F,TaskId) {
    fn into_task(self)
        -> TaskBuild<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Normal,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
    fn into_exit_task(self)
        -> TaskBuild<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Exit,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
}


// fn(P)->R
impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<Currier<F,(P1,),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for F {
    fn into_task(self)
        -> TaskBuild<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Normal,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
    fn into_exit_task(self)
        -> TaskBuild<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self),
                id: TaskId::NONE,
                kind: Kind::Exit,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
}

impl<F:FnOnce(P1,)->R,P1,R> TaskBuildNew<Currier<F,(P1,),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for (F,TaskId) {
    fn into_task(self)
        -> TaskBuild<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Normal,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
    fn into_exit_task(self)
        -> TaskBuild<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskBuild {
            task: TaskCurrier {
                currier: Currier::from(self.0),
                id: self.1,
                kind: Kind::Exit,
            },
            map: TaskMap(PassthroughMapFn::NULL),
            tofn: OneToOne::ONETOONE,
            phantom: PhantomData,
        }
    }
}


macro_rules! impl_task_build_new {
    ($($P:ident),+) => {
        impl<F:FnOnce($($P),+)->R,$($P),+,R> TaskBuildNew<Currier<F,($($P),+),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for F {
            fn into_task(self)
                -> TaskBuild<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskBuild {
                    task: TaskCurrier {
                        currier: Currier::from(self),
                        id: TaskId::NONE,
                        kind: Kind::Normal,
                    },
                    map: TaskMap(PassthroughMapFn::NULL),
                    tofn: OneToOne::ONETOONE,
                    phantom: PhantomData,
                }
            }
            fn into_exit_task(self)
                -> TaskBuild<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskBuild {
                    task: TaskCurrier {
                        currier: Currier::from(self),
                        id: TaskId::NONE,
                        kind: Kind::Exit,
                    },
                    map: TaskMap(PassthroughMapFn::NULL),
                    tofn: OneToOne::ONETOONE,
                    phantom: PhantomData,
                }
            }
        }

        impl<F:FnOnce($($P),+)->R,$($P),+,R> TaskBuildNew<Currier<F,($($P),+),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for (F,TaskId) {
            fn into_task(self)
                -> TaskBuild<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskBuild {
                    task: TaskCurrier {
                        currier: Currier::from(self.0),
                        id: self.1,
                        kind: Kind::Normal,
                    },
                    map: TaskMap(PassthroughMapFn::NULL),
                    tofn: OneToOne::ONETOONE,
                    phantom: PhantomData,
                }
            }
            fn into_exit_task(self)
                -> TaskBuild<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskBuild {
                    task: TaskCurrier {
                        currier: Currier::from(self.0),
                        id: self.1,
                        kind: Kind::Exit,
                    },
                    map: TaskMap(PassthroughMapFn::NULL),
                    tofn: OneToOne::ONETOONE,
                    phantom: PhantomData,
                }
            }
        }






        // impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<Currier<F, ($($P,)+), R>,PassthroughMapFn<R>,()> for F {
        //     fn into_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, PassthroughMapFn<R>,()> {
        //         TaskBuild (
        //             TaskCurrier {
        //                 currier: Currier::from(self),
        //                 id: TaskId::NONE,
        //                 kind: Kind::Normal,
        //             },
        //             TaskMap::None
        //         )
        //     }
            
        //     fn into_exit_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, PassthroughMapFn<R>,()> {
        //         TaskBuild (
        //             TaskCurrier {
        //                 currier: Currier::from(self),
        //                 id: TaskId::NONE,
        //                 kind: Kind::Exit,
        //             },
        //             TaskMap::None
        //         )
        //     }
        // }


        // impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<Currier<F, ($($P,)+), R>, PassthroughMapFn<R>,()> for (F, TaskId) {
        //     fn into_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, PassthroughMapFn<R>,()> {
        //         TaskBuild (
        //             TaskCurrier {
        //                 currier: Currier::from(self.0),
        //                 id: self.1,
        //                 kind: Kind::Normal,
        //             },
        //             TaskMap::None
        //         )
        //     }
            
        //     fn into_exit_task(self) -> TaskBuild<Currier<F, ($($P,)+), R>, PassthroughMapFn<R>,()> {
        //         TaskBuild (
        //             TaskCurrier {
        //                 currier: Currier::from(self.0),
        //                 id: self.1,
        //                 kind: Kind::Exit,
        //             },
        //             TaskMap::None
        //         )
        //     }
        // }
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
    let t :Box<dyn Task> = Box::new(t.task);
    t.run();

    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.task);
    t.run();

    let s = String::new();
    let f = ||{let _s=s;};

    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.task);
    t.run();
}

#[should_panic]
#[test]
fn test_task_new_panic() {
    let f = |_:i32,_:i32|{};
    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.task);
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
    let mut c1: Box<dyn Task> = Box::new(c1.task);
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
    let mut c8: Box<dyn Task> = Box::new(c8.task);
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
