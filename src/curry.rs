// #![feature(unboxed_closures)]

use std::{any::Any, fmt::Debug, marker::PhantomData};

// #[derive(Debug)]
pub(crate) struct Currier<F,C,R> {
    f: F,
    c: C,
    r: PhantomData<R>,
}

impl<F:FnMut()->R,R> From<F> for Currier<F,(),R> {
    fn from(f: F) -> Self {
       Self {
        f,
        c: (),
        r: PhantomData
       }
    }
}

impl<P1,F:FnMut(P1)->R,R> From<F> for Currier<F,(Option<P1>,),R> {
    fn from(f: F) -> Self {
       Self {
        f,
        c: (None::<P1>,),
        r: PhantomData,
       }
    }
}

#[test]
fn test2() {
    let f = ||{};
    let f1 = |a:i32|{};
    let c1 = Currier::from(f);
    let c2 = Currier::from(f1);
    println!("{:?}",c1.c);
    println!("{:?}",c2.c);
}


pub(crate) trait CallOnce {
    type R;
    fn call_once(self)->Self::R;
}
pub(crate) trait CallMut: CallOnce {
    fn call_mut(&mut self)->Self::R;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
}

pub(crate) trait Call: CallMut {
    fn call(&self)->Self::R;
}

pub(crate) trait CallParam {
    fn set(&mut self, i:usize, value: &dyn Any)->bool;
}


impl<F,R> Call for &Currier<F,(),R>
where
    F: Fn()->R,
{
    fn call(&self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> CallMut for &Currier<F,(),R>
where
    F: Fn()->R,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)()
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        None
    }
}

impl<F,R> CallOnce for &Currier<F,(),R>
where
    F: Fn()->R,
{
    type R = F::Output;

    fn call_once(self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> CallMut for &mut Currier<F,(),R>
where
    F: FnMut()->R,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)()
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        None
    }
}

impl<F,R> CallOnce for &mut Currier<F,(),R>
where
    F: FnMut()->R,
{
    type R = F::Output;
    fn call_once(self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> CallOnce for Currier<F,(),R>
where
    F: FnOnce()->R,
{
    type R = R;
    fn call_once(self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> Call for Currier<F,(),R>
where
    F: Fn()->R,
{
    fn call(&self) -> F::Output {
        (self.f)()
    }
}
impl<F,R> CallMut for Currier<F,(),R>
where
    F: FnMut()->R,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)()
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        None
    }
}


#[cfg(test)]
mod test_0 {
    use super::*;
    #[test]
    fn test0() {
        let mut c = Currier::from(||3);
        c.call();
        c.call_mut();
    }
}



impl<F,P1:Clone,R> Call for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam
{
    fn call(&self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
}

impl<F,P1:Clone,R> CallMut for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam
{
    fn call_mut(&mut self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,R> CallOnce for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
{
    type R = F::Output;

    fn call_once(self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
}

impl<F,P1:Clone,R> CallMut for &mut Currier<F,(Option<P1>,),R>
where
    F: FnMut(P1)->R,
    Self: CallParam
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,R> CallOnce for &mut Currier<F,(Option<P1>,),R>
where
    F: FnMut(P1)->R,
{
    type R = F::Output;
    fn call_once(self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
}


impl<F,P1,R> CallOnce for Currier<F,(Option<P1>,),R>
where
    F: FnOnce(P1)->R,
{
    type R = R;
    fn call_once(self) -> F::Output {
        (self.f)(self.c.0.unwrap())
    }
}

impl<F,P1:Clone,R> CallMut for Currier<F,(Option<P1>,),R>
where
    F: FnMut(P1)->R,
    Self: CallParam,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,R> Call for Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam,
{
    fn call(&self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
}


impl<F,P1:Clone+'static,R> CallParam for Currier<F,(Option<P1>,),R>
{
    fn set(&mut self, i:usize, value: &dyn Any)->bool {
        let Some(p1) = value.downcast_ref::<P1>() else {
            return false;
        };
        self.c.0 = Some(p1.clone());
        true
    }
}


#[cfg(test)]
mod test_1 {
    use super::*;
    #[test]
    fn test1() {
        let mut c = Currier::from(|a:i32|a>3);
        c.call();
        c.call_mut();
    }
}
