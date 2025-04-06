
use std::{any::Any, usize};

use crate::curry::{CallMut, CallOnce, CallParam, Currier};

#[derive(Clone,Copy)]
pub enum TaskKind {
    Normal,
    P1,
    R1,
    Exit,
}

pub(crate) trait Task :
    CallMut<R=Option<Box<dyn Any>>>
{
    fn kind(&self)->TaskKind;
}
#[derive(Clone,Copy)]
pub struct Which {
    pub id: usize,
    pub i: usize,
}
impl Default for Which {
    fn default() -> Self {
        Self { id: usize::MAX, i: usize::MAX }
    }
}
impl Which {
    pub const fn new(id:usize,i:usize)->Self {
        Self { id, i }
    }
    pub const fn is_none(&self)->bool {
        self.id == usize::MAX || self.i == usize::MAX
    }
}

pub struct TaskCurrier<Currier> {
    pub(crate) currier: Currier,
    pub(crate) which: Which,
    pub(crate) kind: TaskKind,
}

impl<T> Task for TaskCurrier<T>
    where
    T: CallMut,
    T::R: 'static
{
    fn kind(&self)->TaskKind {
        self.kind
    }
}

impl<T> CallMut for TaskCurrier<T>
    where
    T: CallMut,
    T::R: 'static
{
    fn call_mut(&mut self)->Self::R {
        let r = self.currier.call_mut();
        if std::mem::size_of::<T::R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        self.currier.as_param_mut()
    }
}

impl<T> CallOnce for TaskCurrier<T>
    where
    T: CallOnce,
    T::R: 'static,
{
    type R = Option<Box<(dyn Any+'static)>>;
    fn call_once(self)->Self::R {
        let r = self.currier.call_once();
        if std::mem::size_of::<T::R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn count(&self)->usize {
        self.currier.count()
    }
}

impl<F:FnMut()->R,R> From<(F,Which,TaskKind)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,which,kind): (F,Which,TaskKind)) -> Self {
        Self {
            currier: Currier::from(f),
            which,
            kind,
        }
    }
}

impl<F:FnMut(P1)->R,P1,R> From<(F,Which,TaskKind)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,which,kind): (F,Which,TaskKind)) -> Self {
        Self {
            currier: Currier::from(f),
            which,
            kind,
        }
    }
}


#[cfg(test)]
mod test {
    use crate::CallMut;
    use super::Task;
    use super::TaskCurrier;
    use super::TaskKind;
    use super::Which;

    #[test]
    fn test_from() {
        let f = ||();
        let task 
            = TaskCurrier::from((f,Which::default(),TaskKind::Normal));
        let f = |a:i32|();
        let task 
            = TaskCurrier::from((f,Which::default(),TaskKind::Exit));
    }
    
    #[test]
    fn test1() {
        let f = ||();
        let mut t = TaskCurrier::from((f,Default::default(),TaskKind::Exit));
        t.call_mut();
        
        let f = ||3;
        let nt = TaskCurrier::from((f.clone(),Default::default(),TaskKind::Normal));
        // let nt : Box<dyn Task<R=_>> = Box::new(nt);
        let nt = Box::new(nt);
        let mut nt : Box<dyn Task> = nt;
        nt.call_mut();
    }
    #[test]
    fn test_c1r1() {
        let mut v = 3;
        let f = ||{v=3;v};
        let v = Some(String::new());
        let postdo = |r:i32|{v.unwrap();};
        let r1:&dyn FnMut()->i32 = &f;
        let r1: Box<dyn FnOnce(i32)> = Box::new(postdo);
        r1(3);

        let mut c1 = TaskCurrier::from((|p:i32|(),Default::default(),TaskKind::Normal));
        let mut c1: Box<dyn Task> = Box::new(c1);
        c1.as_param_mut().map(|e|e.set(0, &5));
        c1.call_mut();
    }


}