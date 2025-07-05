
use std::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering}
};

use crate::curry::{CallOnce, CallParam, Currier};

/// The kind of task.
#[derive(Clone,Copy)]
pub enum Kind {
    /// Only executing
    Normal,
    /// Exits the thread after executing
    Exit,
}

static TASKID:TaskIdGen = TaskIdGen::new();

struct TaskIdGen {
    nexter: AtomicUsize
}
impl TaskIdGen {
    const fn new()->Self {
        Self {
            nexter: AtomicUsize::new(0)
        }
    }
    fn next(&self)->usize {
        self.nexter.fetch_add(1, Ordering::Relaxed)
    }
}

/// gen a task ID
pub fn taskid_next()->usize {
    TASKID.next()
}

pub(crate) trait Task
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>>;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
    fn kind(&self)->Kind;
}

#[derive(Clone,Copy)]
/// Represents a position where a condition occurs.
pub struct Anchor(
    /// The task ID associated with the condition.
    pub usize,
    /// The index offset within the condition set.
    pub usize,
);

impl Anchor {
    #[inline]
    pub const fn id(&self)->usize {
        self.0
    }
    #[inline]
    pub const fn i(&self)->usize {
        self.1
    }
    #[inline]
    pub fn set(&mut self, id:usize, i:usize) {
        self.0 = id;
        self.1 = i;
    }
}

/// The carrier of the task, used to create and invoke its functionality.
pub struct TaskCurrier<Currier> {
    pub(crate) currier: Currier,
    pub(crate) to: Option<Anchor>,
    pub(crate) kind: Kind,
}

impl<T> Task for TaskCurrier<T>
    where
    T: CallOnce,
    T::R: 'static
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
}

/// TaskBuildNew is fond to task with an optonal task ID.
pub trait TaskBuildNew<F> where Self:Sized {
    /// just a normal task, without id.
    fn new(f:F)->(Self,Option<usize>);
    /// just a normal task, with taskid.
    fn with(f:F,id:usize)->(Self,Option<usize>);
}
/// TaskBuildOp is to modify a task with an optional condition and exit.
pub trait TaskBuildOp<Currier> {
    /// Sets the target anchor 指向 (taskid 和 condid).
    fn to(self, to: Anchor)->Self;
    /// Marks the task as an exit task, 
    fn exit(self)->Self;
}

impl<Currier> TaskBuildOp<Currier> for (TaskCurrier<Currier>,Option<usize>) {
    fn to(self, to: Anchor)->Self {
        (
            TaskCurrier {
                currier: self.0.currier,
                to: Some(to),
                kind: self.0.kind,
            },
            self.1
        )
    }

    fn exit(self)->Self {
        (
            TaskCurrier {
                currier: self.0.currier,
                to: self.0.to,
                kind: Kind::Exit,
            },
            self.1
        )
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> TaskBuildNew<F> for TaskCurrier<Currier<F,(),R>> {
    fn new(f: F) -> (Self,Option<usize>) {
        (Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn with(f:F,id:usize)->(Self,Option<usize>) {
        (Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        },Some(id))
    }
}

/// constructs a task without cond
impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<F> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn new(f: F) -> (Self,Option<usize>) {
        (Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn with(f:F,id:usize)->(Self,Option<usize>) {
        (Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        },Some(id))
    }
}

/// constructs a task without cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> TaskBuildNew<F> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn new(f: F) -> (Self,Option<usize>) {
        (Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn with(f:F,id:usize)->(Self,Option<usize>) {
        (Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        },Some(id))
    }
}

#[test]
fn test_task_new() {
    // exit task
    TaskCurrier::new(||()).exit();
    // independent task
    TaskCurrier::new(||());
    // independent task
    TaskCurrier::new(||()).to(Anchor(1,0));
    // task#1 with 1 cond and to task#2
    TaskCurrier::with(||9,1).to(Anchor(2,0));
    // task#2 with 1 cond
    TaskCurrier::with(|i:i32|(),2);
}
    
#[test]
fn test1() {
    let f = ||();
    let mut t = TaskCurrier::new(f).exit();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let mut t = TaskCurrier::new(f);
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let s = String::new();
    let f = ||{let s=s;};
    let fr = ||{};

    let mut t = TaskCurrier::new(f);
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();
}

#[test]
fn test_c1r1() {
    let mut v = 3;
    let f = ||{v=3;v};
    let v = Some(String::new());
    let postdo = |_:i32|{v.unwrap();};
    let _r1: &dyn FnMut()->i32 = &f;
    let r1: Box<dyn FnOnce(i32)> = Box::new(postdo);
    r1(3);

    let c1 = TaskCurrier::new(|_p:i32|());
    let mut c1: Box<dyn Task> = Box::new(c1.0);
    c1.as_param_mut().map(|e|e.set(0, &5));
    c1.run();
}
