
use std::{any::Any, usize};

use crate::curry::{self, Call, CallMut, CallOnce, Currier};

#[derive(Clone,Copy)]
pub enum TaskKind {
    Normal,
    P1,
    R1,
    Exit,
}

pub(crate) trait Task :
    curry::CallMut<R=Option<Box<dyn Any>>>
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
pub struct NormalTask<F,C,R> {
    currier: Currier<F,C,R>,
    which: Which,
}

impl<F,R> Call for NormalTask<F,(),R>
where F:Fn()->R,
    R: 'static
{
    fn call(&self)->Self::R {
        let r = self.currier.call();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
}
impl<F,R> CallMut for NormalTask<F,(),R>
where F:FnMut()->R,
    R: 'static
{
    fn call_mut(&mut self)->Self::R {
        let r = self.currier.call_mut();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        None
    }
}
impl<F,R> CallOnce for NormalTask<F,(),R>
where F:FnOnce()->R,
    R:'static
{
    type R = Option<Box<(dyn Any+'static)>>;
    fn call_once(self)->Self::R {
        let r = self.currier.call_once();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
}

impl <F,R> Task for NormalTask<F,(),R>
where F:Fn()->R,
    R: 'static
{
    fn kind(&self)->TaskKind {
        TaskKind::Normal
    }
}

impl<F:FnMut()->R,R> From<F> for NormalTask<F,(),R> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            which: Default::default(),
        }
    }
}
impl<F:FnMut()->R,R> From<(F,&Which)> for NormalTask<F,(),R> {
    fn from((f,which):(F,&Which)) -> Self {
        Self {
            currier: Currier::from(f),
            which:*which
        }
    }
}

impl<F,P1,R> Call for NormalTask<F,(Option<P1>,),R>
    where
    F:Fn(P1)->R,
    P1: Clone + 'static,
    R: 'static,
{
    fn call(&self)->Self::R {
        let r = self.currier.call();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
}
impl<F,P1,R> CallMut for NormalTask<F,(Option<P1>,),R>
    where
    F:FnMut(P1)->R,
    P1: Clone + 'static,
    R: 'static,
{
    fn call_mut(&mut self)->Self::R {
        let r = self.currier.call_mut();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        self.currier.as_param_mut()
    }
}
impl<F,P1,R> CallOnce for NormalTask<F,(Option<P1>,),R>
where F:FnOnce(P1)->R,
    R:'static
{
    type R = Option<Box<(dyn Any+'static)>>;
    fn call_once(self)->Self::R {
        let r = self.currier.call_once();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
}

impl<F,P1,R> Task for NormalTask<F,(Option<P1>,),R>
where F:Fn(P1)->R,
    P1: Clone + 'static,
    R: 'static
{
    fn kind(&self)->TaskKind {
        TaskKind::Normal
    }
}

impl<F:FnMut(P1)->R,P1,R> From<F> for NormalTask<F,(Option<P1>,),R> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            which: Default::default(),
        }
    }
}

impl<F:FnMut(P1)->R,P1,R> From<(F,&Which)> for NormalTask<F,(Option<P1>,),R> {
    fn from((f,which):(F,&Which)) -> Self {
        Self {
            currier: Currier::from(f),
            which:*which
        }
    }
}

// Task with Params(Condtions)
struct C1task<F,P1,R> {
    carrier:Currier<F,(Option<P1>,),R>,
}
impl<F,P1:Clone+'static,R> Call for C1task<F,P1,R>
where
    F:Fn(P1)->R,
    R: 'static
{
    fn call(&self)->Self::R {
        let r = self.carrier.call();
        if 0 == size_of::<R>() {
            None
        } else {
            Some(Box::new(r))
        }
    }
}
impl<F,P1:Clone+'static,R> CallMut for C1task<F,P1,R>
where
    F:FnMut(P1)->R,
    R: 'static
{
    fn call_mut(&mut self)->Self::R {
        let r = self.carrier.call_mut();
        if 0 == size_of::<R>() {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        self.carrier.as_param_mut()
    }
}
impl<F, P1, R> CallOnce for C1task<F, P1, R>
where
    F: FnOnce(P1) -> R,
    R: 'static,
{
    type R = Option<Box<dyn Any>>;
    fn call_once(self) -> Self::R {
        let r = self.carrier.call_once();
        if std::mem::size_of::<R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
}

impl<F,P1:Clone+'static,R> Task for C1task<F,P1,R>
where
    F:Fn(P1)->R,
    R: 'static
{
    fn kind(&self)->TaskKind {
        TaskKind::P1
    }
}

impl<F,P1,R> From<F> for C1task<F,P1,R>
where F:FnMut(P1)->R
{
    fn from(f: F) -> Self {
        Self { carrier: Currier::from(f) }
    }
}

// Task with Exit
pub(crate) struct ExitTask<F,R> {
    curry: curry::Currier<F,(),R>,
}

impl<F,R> Call for ExitTask<F,R>
where
    F: Fn()->R,
    R: 'static
{
    fn call(&self)->Self::R {
        let r = self.curry.call();
        if 0 == size_of::<R>() {
            None
        } else {
            Some(Box::new(r))
        }
    }
}
impl<F,R> CallMut for ExitTask<F,R>
where
    F:FnMut()->R,
    R: 'static
{
    fn call_mut(&mut self)->Self::R {
        let r = self.curry.call_mut();
        if 0 == size_of::<R>() {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        None
    }
}
impl<F,R> CallOnce for ExitTask<F,R>
where
    F: FnOnce()->R,
    R: 'static
{
    type R = Option<Box<dyn Any>>;
    fn call_once(self)->Self::R {
        let r = self.curry.call_once();
        if 0 == size_of::<R>() {
            None
        } else {
            Some(Box::new(r))
        }
    }
}
impl<F,R> Task for ExitTask<F,R>
where
    F: Fn()->R,
    R: 'static
{
    fn kind(&self)->TaskKind {
        TaskKind::Exit
    }
}

impl<F:Fn()->R,R> From<F> for ExitTask<F,R> {
    fn from(f: F) -> Self {
        ExitTask { curry: Currier::from(f) }
    }
}

#[cfg(test)]
mod test {
    use crate::curry::Call;
    use super::C1task;
    use super::ExitTask;
    use super::NormalTask;
    use super::Task;

    #[test]
    fn test1() {
        let f = ||();
        let t = ExitTask::from(f);
        t.call();
        
        let f = ||3;
        let nt = NormalTask::from(f.clone());
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

        let mut c1 = C1task::from(|p:i32|());
        let mut c1: Box<dyn Task> = Box::new(c1);
        c1.as_param_mut().map(|e|e.set(0, &5));
        c1.call_mut();
    }
}