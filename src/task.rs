
use std::{any::Any, usize};

use crate::curry::{CallOnce, CallParam, Currier};

/// The kind of task.
#[derive(Clone,Copy)]
pub enum Kind {
    /// Only executing
    Normal,
    /// Exits the thread after executing
    Exit,
}

pub(crate) trait Task
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>>;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
    fn kind(&self)->Kind;
}

#[derive(Clone,Copy)]
/// Represents a position where a condition occurs.
pub struct Which {
    /// The task ID associated with the condition.
    pub id: usize,
    /// The index offset within the condition set.
    pub i: usize,
}

impl Which {
    pub const fn new(id:usize,i:usize)->Self {
        Self { id, i }
    }
    pub const fn none()->Self {
        Self { id: usize::MAX, i: usize::MAX }
    }
    pub const fn is_none(&self)->bool {
        self.id == usize::MAX || self.i == usize::MAX
    }
}

/// The carrier of the task, used to create and invoke its functionality.
pub struct TaskCurrier<Currier> {
    pub(crate) currier: Currier,
    pub(crate) to: Which,
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

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<(F,Which,Kind)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,to,kind): (F,Which,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to,
            kind,
        }
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<(F,Which)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,to): (F,Which)) -> Self {
        Self {
            currier: Currier::from(f),
            to,
            kind: Kind::Normal,
        }
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<(F,Kind)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,kind): (F,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Which::none(),
            kind,
        }
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<F> for TaskCurrier<Currier<F,(),R>> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            to: Which::none(),
            kind: Kind::Normal,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<(F,Which,Kind)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,to,kind): (F,Which,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to,
            kind,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<(F,Which)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,to): (F,Which)) -> Self {
        Self {
            currier: Currier::from(f),
            to,
            kind: Kind::Normal,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<(F,Kind)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,kind): (F,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Which::none(),
            kind,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<F> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            to: Which::none(),
            kind: Kind::Normal,
        }
    }
}


/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<(F,Which,Kind)> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn from((f,to,kind): (F,Which,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to,
            kind,
        }
    }
}

/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<(F,Which)> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn from((f,to): (F,Which)) -> Self {
        Self {
            currier: Currier::from(f),
            to,
            kind: Kind::Normal,
        }
    }
}

/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<(F,Kind)> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn from((f,kind): (F,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Which::none(),
            kind,
        }
    }
}

/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<F> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>,),R>> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            to: Which::none(),
            kind: Kind::Normal,
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from() {
        let f = ||();
        let _task 
            = TaskCurrier::from((f,Which::none(),Kind::Normal));
        let f = |_:i32|();
        let _task 
            = TaskCurrier::from((f,Which::none(),Kind::Exit));
    }
    
    #[test]
    fn test1() {
        let f = ||();
        let mut t = TaskCurrier::from((f,Which::none(),Kind::Exit));
        let t :Box<dyn Task> = Box::new(t);
        t.run();

        let mut t = TaskCurrier::from(f);
        let t :Box<dyn Task> = Box::new(t);
        t.run();

        let s = String::new();
        let f = ||{let s=s;};
        let fr = ||{};

        let mut t = TaskCurrier::from(f);
        let t :Box<dyn Task> = Box::new(t);
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

        let c1 = TaskCurrier::from((|_p:i32|(),Which::none(),Kind::Normal));
        let mut c1: Box<dyn Task> = Box::new(c1);
        c1.as_param_mut().map(|e|e.set(0, &5));
        c1.run();
    }
}
