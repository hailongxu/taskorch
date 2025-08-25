//! # `task` module
//! 
//! the minimum scheduled and running unit
//! oritided to design type before submit, TaskNeed
//! which contains the necessary inf for a task
//! 
//! the task's result can be configured passed to task input named by CA
//! 
//! task result can be reoutput to many subresult, and pass the subresult to multi-input
//! and this is 1->N
//! 
//! task inputs can be receive from diffrent task-result
//! and this is N->1
//! 
//! ## Exmaples:
//! 
//! ```rust
//! # use taskorch::{TaskBuildNew};
//! // N->1
//! let task = (|a:i32,b:bool|{}).into_task();
//! let task1 = (||3).into_task().bind_to(task.input_ca::<0>());
//! let task2 = (||true).into_task().bind_to(task.input_ca::<1>());
//! ```
//! 
//! 
//! ```rust
//! # use taskorch::{TaskBuildNew};
//! // 1->N
//! let task1 = (|_:i32|{},1.into()).into_task();
//! let task2 = (|_:bool|{},2.into()).into_task();
//! let task = (|_:i32|3)
//!     .into_task()
//!     .map_tuple_with(|_:i32|(true,9))
//!     .bind_all_to((task2.input_ca::<0>(),task1.input_ca::<0>())); 
//! ```
//! 

use std::{
    any::Any, marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}
};

use crate::{cond::{ArgIdx, CondAddr, Section::Input, TaskId}, curry::{CallOnce, CallParam, Currier}, meta::{TupleAt, TupleCondAddr, TupleOpt}};
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

pub(crate) trait Task
{
    /// returns
    /// None: return nothing means `()`
    fn run(self:Box<Self>)->Box<dyn Any>;
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
    fn run(self:Box<Self>)->Box<dyn Any> {
        let r = self.currier.call_once();
        Box::new(r)
        // if std::any::TypeId::of::<T::R>() == std::any::TypeId::of::<()>() {
        //     TaskResult::Void
        // } else {
        //     TaskResult::DynAny(Box::new(r))
        // }
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

pub struct TaskNeed<C,MapFn,MapR,ToFn>
    where MapR: TupleCondAddr
{
    pub(crate) task: TaskCurrier<C>,
    pub(crate) map: TaskMap<MapFn>,
    pub(crate) tofn: ToFn,
    pub(crate) phantom: PhantomData<MapR>
}

impl<C,MapFn,MapR:TupleCondAddr,ToFn> TaskNeed<C,MapFn,MapR,ToFn> {
    /// get task id from task.
    pub fn id(&self)->TaskId {
        self.task.id
    }
}

#[test]
fn test_pass_through() {
    trait Function {}
    impl<F:FnOnce()> Function for F {}

    fn do_nothing() {}
    struct AA<F:Function>(F);
    impl<F:Function> AA<F> {
        // how does we construct a AA ???
        #[cfg(false)]
        fn test()->Self {
            let a = AA(do_nothing);
            a
        }
        fn test2() {
            let a = AA(do_nothing);
            let AA(_f) = a;
            // error, in dev and compling
            #[cfg(false)]
            let a = Self(do_nothing);
        }
    }
}

// impl<Currier:CallOnce+RofCurrier> TaskNeed<Currier, PassthroughMapFn<Currier::Ret>,()>
impl<F,TC,R,MapFn1,R1,ToFn1> TaskNeed<Currier<F,TC,R>, MapFn1,R1,ToFn1>
    where
    TC: TupleOpt,
    R1:TupleCondAddr,
{
    /// Specifies where the result will be delivered.  
    /// Note:
    /// if the task has no result, unit type `()` is returned as per rust-lang.
    /// 
    /// ## Arguments:
    /// * ca: `CondAddr` - identifying the target cond address.
    /// ## Returns:
    /// * `TaskNeed` - with the target condaddr
    pub fn bind_to<'a>(self, ca:CondAddr<R>)
        -> TaskNeed<
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
        TaskNeed {
            task: self.task,
            map,
            tofn,
            phantom: PhantomData,
        }
    }

    #[deprecated(
        since="0.3.0",
        note = "Use `.bind_to()` instead for strict type check. \
               `to()` will be removed in next release."
    )]
    pub fn to<'a>(self, to: usize, ai: usize) 
        -> TaskNeed<
            Currier<F,TC,R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>,
        >
    {
        warn!("Use .bind_to() instead, the .to() will be removed in next version.");
        debug_assert!(ai <= u8::MAX as usize);
        if ai > u8::MAX as usize {
            error!("The index of cond#{ai} is too large, shoul be <= {}.",u8::MAX);
        }
        self.bind_to(CondAddr::from((TaskId::from(to), Input, ArgIdx::from(ai as u8))))
    }
}

impl<F,TC,R,MapFn1,R1,ToFn1> TaskNeed<Currier<F,TC,R>, MapFn1,R1,ToFn1>
    where
    TC: TupleOpt,
    R1: TupleCondAddr,
{
    /// Transforms a single result into multiple outputs for downstream distribution.
    ///
    /// Enables processing one task's output to produce multiple results that can be
    /// directed to different subsequent tasks. Concurrent processing depends on
    /// runtime resources and scheduling.
    /// 
    /// # Example:
    /// 
    /// ```rust
    /// # use taskorch::TaskBuildNew;
    /// let task  = (||3i32) // return i32
    ///     .into_task()
    ///     .map_tuple_with(|_:i32| // recv i32 
    ///         (8i16,"apple") // and return (i16,&'static str)
    ///     );
    /// ```
    /// 
    /// # Arguments
    /// * `mapfn` - A function that takes the task's result of type `R` and returns a tuple of type `MapR`
    /// * `R` - The result type of the task body
    /// * `MapR` - The output type, which must be a tuple `(T1, T2, ..., Tn)`.
    ///
    /// # Returns
    /// Returns a tuple where each element represents the output of one branch.
    /// And the count of tuple max to 8.
    ///
    /// The output structure is:
    /// ```plaintext
    /// (
    ///    value1,  // Output for the 1st branch
    ///    value2,  // Output for the 2nd branch
    ///    ...
    /// )
    /// ```
    ///
    /// # Type Constraints
    /// - `MapR` must be a tuple type
    /// - Each tuple element corresponds to one downstream processing branch
    pub fn map_tuple_with<'a,MapFn,MapR>(self, mapfn:MapFn)
        -> TaskNeed<
            Currier<F,TC,R>,
            MapFn,
            // impl Fndecl<(R,),MapR>,
            MapR,
            OneToOne::<MapR>,
        >
        where
        MapR: TupleCondAddr,
        MapR::TCA: Default,
        MapFn: Fndecl<(R,),MapR>,
    {
        TaskNeed {
            task: self.task,
            map: TaskMap(mapfn),
            tofn: OneToOne::<MapR>::ONETOONE,
            phantom: PhantomData,
        }
    }
}

impl<F,TC,R,MapFn1,R1> TaskNeed<Currier<F,TC,R>, MapFn1,R1,OneToOne<R1>>
    where
    TC: TupleOpt,
    R1: TupleCondAddr,
{
    /// Specifies the destination for all sub-results of this task.
    ///
    /// This method binds each element of the task's output tuple to a corresponding
    /// target condition address. The binding follows a one-to-one correspondence
    /// between the output elements and the provided condition addresses.
    ///
    /// # Note
    /// If the task produces no results, the unit type `()` is returned as per Rust conventions.
    ///
    /// # Arguments
    /// * `cats` - A tuple of condition addresses `(CondAddr, CondAddr, ...)`
    ///   * Each element in the tuple corresponds to exactly one output type in `MapR`
    ///   * The size of this tuple must match the arity of the task's output type `MapR`
    ///   * Each condition address identifies the destination for its corresponding sub-result
    ///
    /// # Returns
    /// * `Self` - The modified task builder with destination addresses configured
    ///
    /// # Example
    /// ```rust
    /// # use taskorch::{Pool, Queue, TaskBuildNew};
    /// # let mut pool = Pool::new();
    /// # let qid = pool.insert_queue(&Queue::new()).unwrap();
    /// # let submitter = pool.task_submitter(qid).unwrap();
    /// // the 1st task#1 receives cond i16 from task
    /// let task1 = (|_:i16|{}).into_task(); 
    /// // the 2nd task#2 receives cond &'static str from task
    /// let task2 = (|_:&'static str|{}).into_task();
    /// // create task with i32 result
    /// let task  = (||3i32).into_task() // the task main body with return type i32
    ///     .map_tuple_with(|_:i32|( // i32 -> (i16, &'static str)
    ///         8i16,    // the 1st branch output
    ///         "apple", // the 2nd branch output
    ///     ));
    /// let task1 = submitter.submit(task1).unwrap();
    /// let task2 = submitter.submit(task2).unwrap();
    /// // bind the result to task1.p0 and task2.p0
    /// let task = task.bind_all_to((
    ///     task1.input_ca::<0>(),
    ///     task2.input_ca::<0>(),
    /// ));
    /// let task = submitter.submit(task);
    /// assert!(task.is_ok());
    /// ```
    pub fn bind_all_to(mut self, cats: R1::TCA)->Self {
        self.tofn.0 = cats;
        self
    }
}

impl<F,TC,R,MapFn1,R1> TaskNeed<Currier<F,TC,R>, MapFn1,R1,OneToOne<R1>>
    where
    TC: TupleOpt,
    R1: TupleCondAddr,
{
    /// Returns a `CondAddr` representing the `I`-th input parameter of this task.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use taskorch::{TaskBuildNew, TaskId};
    ///
    /// // Task without an explicit TaskId (defaults to `NONE`)
    /// let task = (|_: i32, _: bool| {}).into_task();
    /// let ca0 = task.input_ca::<0>();
    /// let ca1 = task.input_ca::<1>();
    /// println!("cond#0 of task: {ca0:?}");
    /// println!("cond#1 of task: {ca1:?}");
    ///
    /// // Task with explicit TaskId = 1
    /// let task = (|_: i32, _: bool| {}, TaskId::from(1)).into_task();
    /// let ca0 = task.input_ca::<0>();
    /// let ca1 = task.input_ca::<1>();
    /// println!("cond#0 of task: {ca0:?}");
    /// println!("cond#1 of task: {ca1:?}");
    /// ```
    ///
    /// # Type Parameters
    /// - `I`: zero-based input-parameter index (`u8`).
    ///
    /// # Returns
    /// `CondAddr<TC::EleT>` locating the `I`-th input.
    ///
    /// # Note
    /// If no `TaskId` is explicitly provided, the address will remain incomplete
    /// until the task is submitted.
    pub fn input_ca<const I:u8>(&self)->CondAddr<TC::EleT>
        where TC: TupleAt<I>
    {
        CondAddr::from((self.id(), Input, ArgIdx::from(I)))
    }
    // at present, not exposed to caller.
    #[doc(hidden)]
    pub fn output_ca<const I:u8>(&self)->CondAddr<R1::EleT>
        where R1: TupleAt<I>
    {
        use crate::cond::Section;
        CondAddr::from((self.id(), Section::Output, ArgIdx::from(I)))
    }
}

// Internal use only
// just for keep the origin type of task input
// TODO: maybe will be merged into CallOnce, at present, use this
#[doc(hidden)]
pub trait PsOf {
    type InputPs;
}

impl<F,C:TupleOpt,R> PsOf for Currier<F,C,R> {
    type InputPs = C;
}


/// TaskBuildOp provides target condaddr configuration.
#[deprecated(
    since="0.3.0",
    note = "Use `.bind_to()` directly, for this method has been integrated into the TaskNeed. \
           trait `TaskBuildOp` actually do nothing and will be removed in next release."
)]
pub trait TaskBuildOp<Currier,R> {}

/// A builder trait for constructing tasks with an optional task ID.
pub trait TaskBuildNew<C,F,R,T> {
    /// construct a task from a function or a closure or with an taskid.
    /// 
    /// # Example:
    /// ```rust
    /// # use taskorch::TaskBuildNew;
    /// // no return
    /// let task = (||{}).into_task(); // just a function
    /// let task = (||{},1.into()).into_task(); // function and an explicit taskid
    /// let task = (|_:i32|{}).into_task(); // task with one cond
    /// let task = (|_:i32|{},2.into()).into_task(); // task with one cond and explicit taskid
    /// // with return
    /// let task = (||3).into_task(); // just a function
    /// let task = (||3,1.into()).into_task(); // function and an explicit taskid
    /// let task = (|_:i32|3).into_task(); // task with one cond
    /// let task = (|_:i32|3,2.into()).into_task(); // task with one cond and explicit taskid
    /// ```
    /// 
    /// # Arguments:
    /// * (fun,TaskId)
    /// * fun : a function or a closure with param count less equal 8
    /// * taskid: `TaskId`, you can also input the id explicitly
    /// 
    /// A `taskid` is required when the function has parameters, because other tasks
    /// need to know the location `CondAddr(taskid, cond#i)` to which they pass conditions.
    /// If the task has no parameters, the `taskid` is not required.
    /// However, if you omit it, the system will automatically generate a `taskid`.
    /// 
    /// # Returns
    /// 
    /// - TaskNeed: including the necessaary info of a task, 
    ///   just for preparetion of submition.
    fn into_task(self)->TaskNeed<C,F,R,T> where R: TupleCondAddr;

    #[deprecated(
        since="0.2.0",
        note = "Use `into_task()` instead for clearer ownership semantic. \
               `task()` will be removed in next release."
    )]
    fn task(self)->TaskNeed<C,F,R,T> where Self:Sized, R: TupleCondAddr {
        self.into_task()
    }

    /// construct a exit task
    /// # Note
    /// This is functionally identical to `into_task()`, with the additional behavior of thread exit gracefully
    /// after task completion.
    fn into_exit_task(self)->TaskNeed<C,F,R,T> where R:TupleCondAddr;

    #[deprecated(
        since="0.2.0",
        note = "Use `into_exit_task()` instead for clearer ownership semantic. \
               `exit_task()` will be removed in next release."
    )]
    fn exit_task(self)->TaskNeed<C,F,R,T> where Self:Sized, R:TupleCondAddr {
        self.into_exit_task()
    }
}


#[test]
fn test_task_build_fan_and_to() {
    let task = (||3).into_task();
    if true {
        task.map_tuple_with(|_:i32| (3,));
    } else {
        task.bind_to(CondAddr::from((TaskId::from(3),Input, ArgIdx::<i32>::AI0)));
    }

    let task = (||{}).into_task();
    match 0 {
        0 => {task.map_tuple_with(|_:()| (3,) ); }
        1 => {task.bind_to(CondAddr::from((TaskId::from(3),Input, ArgIdx::AI0))); }
        2 => {task.to(1,2); }
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

// Internal use only
#[doc(hidden)]
pub struct OneToOne<Rtuple:TupleCondAddr>(Rtuple::TCA);
impl<P:TupleCondAddr> OneToOne<P>
{
    const ONETOONE:Self = Self(P::ONETOONE);
}
impl<'a,Rtuple:TupleCondAddr> Fndecl<(&'a Rtuple,),Rtuple::TCA> for OneToOne<Rtuple> {
    type Pt = (&'a Rtuple,);
    type R = Rtuple::TCA;
    fn call(self,_ps:Self::Pt)->Self::R {
        self.0
    }
}
/// constructs a task without cond
impl<F:FnOnce()->R,R> TaskBuildNew<Currier<F,(),R>,PassthroughMapFn<R>,(R,),OneToOne<(R,)>> for F {
    fn into_task(self)
        -> TaskNeed<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
        -> TaskNeed<
            Currier<F,(P1,),R>,
            PassthroughMapFn<R>,
            (R,),
            OneToOne<(R,)>
        >
        where
        PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
    {
        TaskNeed {
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
                -> TaskNeed<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskNeed {
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
                -> TaskNeed<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskNeed {
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
                -> TaskNeed<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskNeed {
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
                -> TaskNeed<
                    Currier<F,($($P),+),R>,
                    PassthroughMapFn<R>,
                    (R,),
                    OneToOne<(R,)>
                >
                where
                PassthroughMapFn<R>: Fndecl<(R,),(R,)>,
            {
                TaskNeed {
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
fn test_taskneed_construct() {
    let task: TaskNeed<Currier<_, (), ()>, PassthroughMapFn<()>, ((),), OneToOne<((),)>>
        = (||println!("task='free':  Hello, 1 2 3 .."),TaskId::from(1)).into_task();
    println!("task type={:?}", std::any::type_name_of_val(&task));
}

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
    let r = c8.run();
    let r = r.downcast::<i32>().unwrap();
    assert_eq!(*r, tr8);
}
