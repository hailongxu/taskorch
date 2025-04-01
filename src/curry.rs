// #![feature(unboxed_closures)]

use std::marker::PhantomData;

// #[derive(Debug)]
pub(crate) struct Currier<F,C,R> {
    f: F,
    c: C,
    r: PhantomData<R>,
}

impl<F:Fn()->R,R> From<F> for Currier<F,(),R> {
    fn from(f: F) -> Self {
       Self {
        f,
        c: (),
        r: PhantomData
       }
    }
}

impl<P1,F:Fn(P1)->R,R> From<F> for Currier<F,(Option<P1>,),R> {
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
}

pub(crate) trait Call: CallMut {
    fn call(&self)->Self::R;
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
}


#[cfg(test)]
mod test_0 {
    use super::*;
    #[test]
    fn test1() {
        let mut c = Currier::from(||3);
        c.call();
        c.call_mut();
    }
}



impl<F,P1:Clone,R> Call for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
{
    fn call(&self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
}

impl<F,P1:Clone,R> CallMut for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
{
    fn call_mut(&mut self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
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
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
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
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
}

impl<F,P1:Clone,R> Call for Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
{
    fn call(&self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
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
