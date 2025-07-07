
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
pub trait TaskBuildNew<TC> {
    /// just a normal task, without id.
    fn task(self)->(TC,Option<usize>);
    fn exit_task(self)->(TC,Option<usize>);
}
/// TaskBuildOp is to modify a task with an optional condition and exit.
pub trait TaskBuildOp<Currier> {
    /// Sets the target anchor 指向 (taskid 和 condid).
    fn to(self, to: Anchor)->Self;
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
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> TaskBuildNew<TaskCurrier<Currier<F,(),R>>> for F {
    fn task(self) -> (TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn exit_task(self)->(TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Exit,
        },None)
    }
}
impl<F:FnOnce()->R,R> TaskBuildNew<TaskCurrier<Currier<F,(),R>>> for (F,usize) {
    fn task(self) -> (TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Normal,
        },Some(self.1))
    }
    fn exit_task(self) -> (TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Exit,
        },Some(self.1))
    }
}

impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<TaskCurrier<Currier<F,(Option<P1>,),R>>> for F {
    fn task(self) -> (TaskCurrier<Currier<F,(Option<P1>,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn exit_task(self) -> (TaskCurrier<Currier<F,(Option<P1>,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Exit,
        },None)
    }
}
impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<TaskCurrier<Currier<F,(Option<P1>,),R>>> for (F,usize) {
    fn task(self) -> (TaskCurrier<Currier<F,(Option<P1>,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Normal,
        },Some(self.1))
    }
    fn exit_task(self) -> (TaskCurrier<Currier<F,(Option<P1>,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Exit,
        },Some(self.1))
    }
}

macro_rules! impl_task_build_new {
    ($($P:ident),+) => {
        impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<TaskCurrier<Currier<F, ($(Option<$P>,)+), R>>> for F {
            fn task(self) -> (TaskCurrier<Currier<F, ($(Option<$P>,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self),
                    to: None,
                    kind: Kind::Normal,
                }, None)
            }
            
            fn exit_task(self) -> (TaskCurrier<Currier<F, ($(Option<$P>,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self),
                    to: None,
                    kind: Kind::Exit,
                }, None)
            }
        }


        impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<TaskCurrier<Currier<F, ($(Option<$P>,)+), R>>> for (F, usize) {
            fn task(self) -> (TaskCurrier<Currier<F, ($(Option<$P>,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self.0),
                    to: None,
                    kind: Kind::Normal,
                }, Some(self.1))
            }
            
            fn exit_task(self) -> (TaskCurrier<Currier<F, ($(Option<$P>,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self.0),
                    to: None,
                    kind: Kind::Exit,
                }, Some(self.1))
            }
        }
    };
}

impl_task_build_new!(P1, P2);



#[test]
fn test_task_new() {
    let f = ||();
    let t = f.exit_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let t = f.task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let s = String::new();
    let f = ||{let _s=s;};

    let t = f.task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let f = |_:i32,_:i32|{};
    let t = f.task();
}

#[test]
fn test_task_run() {
    let mut v = 3;
    let f = ||{v=3;v};
    let v = Some(String::new());
    let postdo = |_:i32|{v.unwrap();};
    let _r1: &dyn FnMut()->i32 = &f;
    let r1: Box<dyn FnOnce(i32)> = Box::new(postdo);
    r1(3);

    let c1 = (|_p:i32|()).task();
    let mut c1: Box<dyn Task> = Box::new(c1.0);
    c1.as_param_mut().map(|e|e.set(0, &5));
    c1.run();
}
