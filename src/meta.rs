
use crate::cond::CondAddr;


#[doc(hidden)]
pub trait TupleAt<const I:u8> {
    type EleT;
    fn value_at(&self)->&Self::EleT;
}
impl<const I:u8> TupleAt<I> for () {
    type EleT = ();
    fn value_at(&self)->&() {
        &()
    }
}
impl<T1> TupleAt<0> for (T1,) {
    type EleT=T1;
    fn value_at(&self)->&Self::EleT {
        &self.0
    }
}

trait TupleExtAt {
    fn at<const I:u8>(&self)->&<Self as TupleAt<I>>::EleT
        where Self: TupleAt<I>;
}

impl<T> TupleExtAt for T {
    fn at<const I:u8>(&self)->&<Self as TupleAt<I>>::EleT
        where Self: TupleAt<I> {
            <Self as TupleAt<I>>::value_at(&self)
    }
}


// // error
// macro_rules! tuple_at_impl {
//     ($($i:tt $T:ident),+) => {
//         $(
//         impl<$($T),+> TupleAt<$i> for ($($T),+) {
//             type T=$T;
//             fn value_at(&self)->&Self::T {
//                 &self.$i
//             }
//         }
//         )+
//     };
// }
// tuple_at_impl!(0 T1,1 T2);

macro_rules! tuple_at_impl {
    ($i: tt $TO: ident; $($T: ident),+) => {
        impl<$($T),+> TupleAt<$i> for ($($T),+) {
            type EleT=$TO;
            fn value_at(&self)->&Self::EleT {
                &self.$i
            }
        }
    };
}

// macro_rules! tuple_at_impl2 {
//     ($($n:tt $T:ident),+;$($Ti:ident),+) => {
//         $(
//         tuple_at_impl!($n $T; $($Ti) +);
//         )+
//     };
// }

// tuple_at_impl2!(0 T1,1 T2; T1,T2);

tuple_at_impl!(0 T1; T1,T2);
tuple_at_impl!(1 T2; T1,T2);

tuple_at_impl!(0 T1; T1,T2,T3);
tuple_at_impl!(1 T2; T1,T2,T3);
tuple_at_impl!(2 T3; T1,T2,T3);

tuple_at_impl!(0 T1; T1,T2,T3,T4);
tuple_at_impl!(1 T2; T1,T2,T3,T4);
tuple_at_impl!(2 T3; T1,T2,T3,T4);
tuple_at_impl!(3 T4; T1,T2,T3,T4);

tuple_at_impl!(0 T1; T1,T2,T3,T4,T5);
tuple_at_impl!(1 T2; T1,T2,T3,T4,T5);
tuple_at_impl!(2 T3; T1,T2,T3,T4,T5);
tuple_at_impl!(3 T4; T1,T2,T3,T4,T5);
tuple_at_impl!(4 T5; T1,T2,T3,T4,T5);

tuple_at_impl!(0 T1; T1,T2,T3,T4,T5,T6);
tuple_at_impl!(1 T2; T1,T2,T3,T4,T5,T6);
tuple_at_impl!(2 T3; T1,T2,T3,T4,T5,T6);
tuple_at_impl!(3 T4; T1,T2,T3,T4,T5,T6);
tuple_at_impl!(4 T5; T1,T2,T3,T4,T5,T6);
tuple_at_impl!(5 T6; T1,T2,T3,T4,T5,T6);

tuple_at_impl!(0 T1; T1,T2,T3,T4,T5,T6,T7);
tuple_at_impl!(1 T2; T1,T2,T3,T4,T5,T6,T7);
tuple_at_impl!(2 T3; T1,T2,T3,T4,T5,T6,T7);
tuple_at_impl!(3 T4; T1,T2,T3,T4,T5,T6,T7);
tuple_at_impl!(4 T5; T1,T2,T3,T4,T5,T6,T7);
tuple_at_impl!(5 T6; T1,T2,T3,T4,T5,T6,T7);
tuple_at_impl!(6 T7; T1,T2,T3,T4,T5,T6,T7);

tuple_at_impl!(0 T1; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(1 T2; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(2 T3; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(3 T4; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(4 T5; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(5 T6; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(6 T7; T1,T2,T3,T4,T5,T6,T7,T8);
tuple_at_impl!(7 T8; T1,T2,T3,T4,T5,T6,T7,T8);

#[test]
fn test_tuple_at() {
    type T = (i32,&'static str);
    let a = <T as TupleAt::<0>>::value_at(&(2,""));
    let a = (2,"").at::<0>();
}




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


pub trait Fndecl<PS,R> {
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
    {
        fn get<F:Fndecl<(String,),(String,)>>(_f:F) {}
        fn ff2(_:String)->(String,) {(String::new(),)}
        get(ff2);
        struct AA<F:Fndecl<(String,),(String,)>> {
            f: F,
        }
        let aa = AA {
            f: ff2,
        };
    }
    {
        fn get<F:Fndecl<Pt,R>,Pt,R>(_f:F) {}
        fn ff() {}
        fn ff2(_:String)->(String,) {(String::new(),)}
        get(ff);
        get(ff2);
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
}

pub trait TupleCondAddr {
    type E1;
    type TCA; // CondAddrTuple
    const ONETOONE: Self::TCA;
}

impl TupleCondAddr for () {
    type E1 = ();
    type TCA = ();
    const ONETOONE: Self::TCA = ();
}

// pub(crate) struct Single<T>(PhantomData<T>);
// impl<T> TupleCondAddr for Single<T> {
//     type E1 = T;
//     type Cat = CondAddr<T>;
//     // const NONE:Self::Opt = (None::<T1>,);
// }

impl<T1> TupleCondAddr for (T1,) {
    type E1 = T1;
    type TCA = (CondAddr<T1>,);
    const ONETOONE: Self::TCA = (CondAddr::<T1>::new::<0>(),);
}

macro_rules! impl_tuple_condaddr {
    ($($n:literal $T:ident),+) => {
        impl<$($T),+> TupleCondAddr for ($($T,)+) {
            type E1 = T1;
            type TCA = ($(CondAddr<$T>,)+);
            const ONETOONE:Self::TCA = ($(CondAddr::<$T>::new::<$n>(),)+);
        }
    };
}

impl_tuple_condaddr!(0 T1,1 T2);
impl_tuple_condaddr!(0 T1,1 T2, 2 T3);
impl_tuple_condaddr!(0 T1,1 T2, 2 T3,3 T4);
impl_tuple_condaddr!(0 T1,1 T2, 2 T3,3 T4,4 T5);
impl_tuple_condaddr!(0 T1,1 T2, 2 T3,3 T4,4 T5,5 T6);
impl_tuple_condaddr!(0 T1,1 T2, 2 T3,3 T4,4 T5,5 T6,6 T7);
impl_tuple_condaddr!(0 T1,1 T2, 2 T3,3 T4,4 T5,5 T6,6 T7,7 T8);


#[test]
fn test_tuple_condaddr() {
    use crate::cond::{TaskId,ArgIdx,Place};
    let addr = <(i32, u32) as TupleCondAddr>::ONETOONE;
    assert_eq!(addr.0, CondAddr::<i32>::from((TaskId::NONE,Place::Input,ArgIdx::AI0)));
    assert_eq!(addr.1, CondAddr::<u32>::from((TaskId::NONE,Place::Input,ArgIdx::AI1)));
    dbg!(addr);
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

#[test]
fn test_type_check() {
    use std::marker::PhantomData;

    struct TypeChecker<T> {
        _marker: PhantomData<T>,
    }

    impl<T> TypeChecker<T> {
        const IS_UNIT: bool = std::mem::size_of::<T>() == std::mem::size_of::<()>()
            && std::mem::align_of::<T>() == std::mem::align_of::<()>();
    }

    struct AA;
    struct BB;
    println!("i32 is i32: {}", TypeChecker::<AA>::IS_UNIT);    // true
    println!("f64 is i32: {}", TypeChecker::<f64>::IS_UNIT);    // false

    struct IsType<A,B> {
        a: PhantomData<A>,
        b: PhantomData<B>,
    }

    impl<A,B> IsType<A,B>  {
        // add the fellowed line to the top of file, in unstable rust version.
        #![feature(specialization)]
        #[cfg(false)]
        default const SAME: bool = false;
    }
    impl<T> IsType<T,T>  {
        const SAME: bool = true;
    }

    println!("i32,i32 :{}",IsType::<i32,i32>::SAME/*,IsType::<i32,i32>::SAME1*/);
    // error `SAME` does not exist under IsType::<i32,u32>
    #[cfg(false)]
    println!("i32,u32 :{} {}",IsType::<i32,u32>::SAME,IsType::<i32,i32>::SAME1);

    fn is_same_type<A:'static,B:'static>() -> bool {
        std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>()
    }

    assert!(is_same_type::<(),()>());
    assert!(!is_same_type::<(),AA>());
}