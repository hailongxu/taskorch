
use crate::curry::{self, Call,CallMut,CallOnce, Currier};

#[derive(Clone,Copy)]
pub enum TaskKind {
    Normal,
    P1,
    R1,
    Exit,
}

pub(crate) trait Task : curry::Call {
    fn kind(&self)->TaskKind;
}

pub struct NormalTask<F,R> {
    currier: Currier<F,(),R>,
}

impl<F,R> Call for NormalTask<F,R>
where F:Fn()->R
{
    fn call(&self) {
        self.currier.call();
    }
}
impl<F,R> CallMut for NormalTask<F,R>
where F:FnMut()->R
{
    fn call_mut(&mut self) {
        self.currier.call_mut();
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        None
    }
}
impl<F,R> CallOnce for NormalTask<F,R>
where F:FnOnce()->R
{
    type R = ();
    fn call_once(self) {
        self.currier.call_once();
    }
}

impl <F,R> Task for NormalTask<F,R>
where F:Fn()->R
{
    fn kind(&self)->TaskKind {
        TaskKind::Normal
    }
}

impl<F:Fn()->R,R> From<F> for NormalTask<F,R> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
        }
    }
}


// Task with Params(Condtions)
struct C1task<F,P1,R> {
    carrier:Currier<F,(Option<P1>,),R>,
}
impl<F,P1:Clone+'static,R> Call for C1task<F,P1,R>
where F:Fn(P1)->R
{
    fn call(&self) {
        self.carrier.call();
    }
}
impl<F,P1:Clone+'static,R> CallMut for C1task<F,P1,R>
where F:FnMut(P1)->R
{
    fn call_mut(&mut self) {
        self.carrier.call_mut();
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        self.carrier.as_param_mut()
    }
}
impl<F,P1,R> CallOnce for C1task<F,P1,R>
where F:FnMut(P1)->R
{
    type R = ();
    fn call_once(self) {
        self.carrier.call_once();
    }
}

impl<F,P1:Clone+'static,R> Task for C1task<F,P1,R>
where F:Fn(P1)->R
{
    fn kind(&self)->TaskKind {
        TaskKind::P1
    }
}

impl<F,P1,R> From<F> for C1task<F,P1,R>
where F:Fn(P1)->R
{
    fn from(f: F) -> Self {
        Self { carrier: Currier::from(f) }
    }
}

// Task with Return
struct TaskC1<F,R,Do> {
    currier:Currier<F,(),R>,
    taskid: usize,
    ci: usize,
    process: Do,
}

impl<F,R,Do:FnMut(R)> Call for TaskC1<F,R,Do>
where F:Fn()->R
{
    fn call(&self) {
        panic!("should not be come here");
    }
}

impl<F,R,Do:FnMut(R)> CallMut for TaskC1<F,R,Do>
where F:FnMut()->R
{
    fn call_mut(&mut self) {
        let r = self.currier.call_mut();
        (self.process)(r);
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        None
    }
}
impl<F,R,Do:FnMut(R)> CallOnce for TaskC1<F,R,Do>
where F:FnOnce()->R
{
    type R=();
    fn call_once(mut self) {
        let r = self.currier.call_once();
        (self.process)(r);
    }
}

impl<F,R,Do> Task for TaskC1<F,R,Do>
where
    F:Fn()->R,
    Do:FnMut(R),
{
    fn kind(&self)->TaskKind {
        TaskKind::R1
    }
}

impl<F,R,Do> From<(F,Do,usize,usize)> for TaskC1<F,R,Do>
where
    F:Fn()->R,
    Do:FnMut(R),
{
    fn from((f,handle,targetid,ci): (F,Do,usize,usize)) -> Self {
        Self {
            currier: Currier::from(f),
            process: handle,
            taskid: targetid,
            ci,
        }
    }
}


// Task with Exit
pub(crate) struct ExitTask<F,R> {
    curry: curry::Currier<F,(),R>,
}

impl<F:Fn()->R,R> Call for ExitTask<F,R> {
    fn call(&self) {
        self.curry.call();
    }
}
impl<F:FnMut()->R,R> CallMut for ExitTask<F,R> {
    fn call_mut(&mut self) {
        self.curry.call_mut();
    }
    fn as_param_mut(&mut self)->Option<&mut dyn curry::CallParam> {
        None
    }
}
impl<F:FnOnce()->R,R> CallOnce for ExitTask<F,R> {
    type R = ();
    fn call_once(self) {
        self.curry.call_once();
    }
}
impl<F:Fn()->R,R> Task for ExitTask<F,R> {
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
    use super::TaskC1;

    #[test]
    fn test1() {
        let f = ||();
        let t = ExitTask::from(f);
        t.call();
        
        let f = ||3;
        let nt = NormalTask::from(f.clone());
        // let nt : Box<dyn Task<R=_>> = Box::new(nt);
        let nt = Box::new(nt);
        let nt : Box<dyn Task<R=_>> = nt;
        nt.call();
    }
    #[test]
    fn test_c1r1() {
        let mut saved = 0; 
        let r1 = TaskC1::from((||3,|r|saved=r,0,0));
        let mut r1: Box<dyn Task<R=()>> = Box::new(r1);

        let c1 = C1task::from(|p:i32|dbg!(p));
        let mut c1: Box<dyn Task<R=()>> = Box::new(c1);
        c1.as_param_mut().map(|e|e.set(0, &5));

        r1.call_mut();
        c1.call();
    }
}