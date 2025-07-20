

pub(crate) trait Identical<T> {}
impl<T> Identical<T> for T {}

pub(crate) trait TupleOpt {
    type Opt;
    const NONE: Self::Opt;
}

impl TupleOpt for () {
    type Opt = ();
    const NONE:Self::Opt = ();
}

impl<T1> TupleOpt for (T1,) {
    type Opt = (Option<T1>,);
    const NONE:Self::Opt = (None::<T1>,);
}

macro_rules! impl_tupleopt {
    ($($T:ident),+) => {
        impl<$($T),+> TupleOpt for ($($T,)+) {
            type Opt = ($(Option<$T>,)+);
            const NONE:Self::Opt = ($(None::<$T>,)+);
        }
    };
}

impl_tupleopt!(T1,T2);
impl_tupleopt!(T1,T2,T3);
impl_tupleopt!(T1,T2,T3,T4);
impl_tupleopt!(T1,T2,T3,T4,T5);
impl_tupleopt!(T1,T2,T3,T4,T5,T6);
impl_tupleopt!(T1,T2,T3,T4,T5,T6,T7);
impl_tupleopt!(T1,T2,T3,T4,T5,T6,T7,T8);


pub(crate) trait Fndecl<PS,R> {
    type Pt;// Params Tuple : From<PS>;
    type R;
    fn call(self,ps:Self::Pt)->Self::R;
}

impl<F,R> Fndecl<(),R> for F
    where F:FnOnce()->R
{
    type Pt = ();
    type R = R;
    fn call(self,_ps:Self::Pt)->Self::R {
        self()
    }
}

impl<F,P1,R> Fndecl<(P1,),R> for F
    where F:FnOnce(P1)->R
{
    type Pt = (P1,);
    type R = R;
    fn call(self,ps:Self::Pt)->Self::R {
        self(ps.0)
    }
}

macro_rules! fndecl_impl {
    ($($n:tt $P:ident),+) => {
        impl<F,$($P),+,R> Fndecl<($($P),+),R> for F
            where F:FnOnce($($P),+)->R
        {
            type Pt = ($($P),+);
            type R = R;
            fn call(self,ps:Self::Pt)->Self::R {
                self($(ps.$n),+)
            }
        }
    };
}

fndecl_impl!(0 P1, 1 P2);
fndecl_impl!(0 P1, 1 P2, 2 P3);
fndecl_impl!(0 P1, 1 P2, 2 P3, 3 P4);
fndecl_impl!(0 P1, 1 P2, 2 P3, 3 P4, 4 P5);
fndecl_impl!(0 P1, 1 P2, 2 P3, 3 P4, 4 P5, 5 P6);
fndecl_impl!(0 P1, 1 P2, 2 P3, 3 P4, 4 P5, 5 P6, 6 P7);
fndecl_impl!(0 P1, 1 P2, 2 P3, 3 P4, 4 P5, 5 P6, 6 P7, 7 P8);


#[test]
fn test_fndecl() {
    fn get<F:Fndecl<Pt,R>,Pt,R>(_f:F) {}
    get(||3);
    get(|_:i8|3);
    get(|_:i8,_:i8,|3);
    get(|_:i8,_:i8,_:i8|3);
    get(|_:i8,_:i8,_:i8,_:i8,|3);
    get(|_:i8,_:i8,_:i8,_:i8,_:i8|3);
    get(|_:i8,_:i8,_:i8,_:i8,_:i8,_:i8,|3);
    get(|_:i8,_:i8,_:i8,_:i8,_:i8,_:i8,_:i8|3);
    get(|_:i8,_:i8,_:i8,_:i8,_:i8,_:i8,_:i8,_:i8|3);
}

pub(crate) trait Handle {
    // type T;
    fn handle<T>(&self,i:usize,t:&T);
}
trait TupleEachDo {
    fn foreach(&self,each_do:impl Handle);
}

impl TupleEachDo for () {
    fn foreach(&self,_each_do:impl Handle) {
    }
}

impl<T1> TupleEachDo for (T1,) {
    fn foreach(&self,each_do:impl Handle) {
        each_do.handle(0,&self.0);
    }
}
impl<T1,T2> TupleEachDo for (T1,T2) {
    fn foreach(&self,each_do:impl Handle) {
        each_do.handle(0,&self.0);
        each_do.handle(1,&self.1);
    }
}

#[cfg(test)]
mod test_tuple {
    use std::{any::type_name_of_val, fmt::Debug};
    use super::*;

    fn handle<T>(i:usize,t:T) {
        println!("handle #{i} {}", type_name_of_val(&t));
    }
    struct A;
    impl Handle for A {
        fn handle<T>(&self,i:usize,t:&T) {
           handle(i,t);
        }
    }

    #[test]
    fn test_foreach() {
        (3,).foreach(A);
        (3,"ss").foreach(A);
    }
}


#[cfg(test)]
mod test_tuple2 {
    trait Handle {
        fn handle<T>(&self,t:T);
    }
    trait TupleDo<Do> {
        fn feach(&self,handle:Do);
    }
    impl<Do:Handle> TupleDo<Do> for () {
        fn feach(&self,handle:Do) {
        }
    }
    impl<Do:Handle,T1> TupleDo<Do> for (T1,) {
        fn feach(&self,handle:Do) {
            handle.handle(&self.0);
        }
    }
    impl<Do:Handle,T1,T2> TupleDo<Do> for (T1,T2) {
        fn feach(&self,handle:Do) {
            handle.handle(&self.0);
            handle.handle(&self.0);
        }
    }
    
}


#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    pub(crate) trait WhenTupleComed {
        fn foreach(&self);
    }
    impl<T:'static+Debug> WhenTupleComed for ((T,usize),) {
        fn foreach(&self) {
            when_ci_comed(&self.0.0, &self.0.1);
        }
    }

    fn when_ci_comed<T:Debug>(t:&T, i:&usize) {
        println!("----{t:?} {i}----");
    }


    struct C<A,B>(A,B);
    fn get<F:Fndecl<P,R>,P,R,F2:Fndecl<(F::R,),R2>,R2>(f:F,pp:F::Pt,f2:F2)
    where F2::R : WhenTupleComed,
    F2::Pt: From<(F::R,)>,
    F2::Pt: Identical<(F::R,)>
    {
        let f = f;
        let c = C(f,f2);
        let r = c.0.call(pp);
        let r2 = c.1.call((r,).into());
        r2.foreach();
    }

    #[test]
    fn test_fndel() {
        struct A;
        let a = A;
        let f = ||{drop(a);3};
        let d = |_:i32|((3,3usize),);
        get(f,(),d);
        // get(|_:i32|9,(8,));
    }
    fn test_fndel0() {
        let f = ||{};
        let d = |_:()|((3,3usize),);
        get(f,(),d);
        // get(|_:i32|9,(8,));
    }
}