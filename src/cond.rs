//! ## `cond` module
//! 
//! Core scheduling concepts:
//! 
//! <1> CondAddr<T>
//! 
//! <1.1> Logical address to locate a condition (not a memory address)
//! Each Cond belongs to task, which is identified/marked/found by taskid.
//! Each task has many Params, which is identified/marked/found by Pi.
//! hence, the condaddr can be unique be located by taskid and paramter index.
//! Contruct a CondAddr via `from()`.
//! 
//! <1.2> Type Annotation: CondAddr's data association requires explicit typing, 
//! defining the Type-annotation concept. CondAddr is inherently type-annotated.
//! 
//! ## Exmaples:
//! ```rust
//! // cond addr is at (Task#1 and Task.Param#0) 
//! let ca = CondAddr::from((TaskId::new(1),Pi::PI0)); 
//! ```
//! 
//! <2> TaskId: Unique identifier for a task, as the first component of CondAddr.
//! 
//! <3> Pi: Zero-based index of a task parameter (also used as cond i), 
//! as the 2nd conponent of CondAddr, whose type is derived from here.
//! 


use std::{any::type_name, marker::PhantomData, num::NonZeroUsize, fmt::Debug};

/// TaskId
/// the unique in given pool instance system
/// 
/// enforce `TaskId` zero/non-zero semantics
/// - Zero `TaskId` is now strictly internal (auto-assigned for unconditional tasks)
/// - Caller attempts to create zero IDs will log warnings
/// - Explicit non-zero IDs are required for all tasks (both conditional and unconditional). 
/// 
/// You can also convert the `TaskId` to `usize`, using `.as_usize()`.
/// 
/// # Example:
/// ```
/// let taskid = TaskId::from(3);
/// let taskid = 3.into();
/// let uid = taskid.as_usize();
/// ```
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct TaskId(pub(crate) Option<NonZeroUsize>);

impl TaskId {
    pub(crate) const NONE : Self = Self(None);


    /// Construct a TaskId from usize，just use in self crate, not expose it.
    /// and does not output any message.
    #[inline]
    pub(crate) const fn new(id:usize)->Self {
        Self(NonZeroUsize::new(id))
    }

    /// convert the TskId to usize
    #[inline]
    pub const fn as_usize(&self)->usize {
        match self.0 {
            Some(v) => v.get(),
            None => 0,
        }
    }
}

/// construct a `TaskId` from a `usize`
///
/// # Behavior
/// - For non-zero IDs: Always succeeds
/// - For zero ID:
///   - In debug mode: panics immediately
///   - In release mode: logs warning but returns `TaskId::NONE`
/// 
/// Examples:
/// ```rust
/// # use taskorch::task::TaskId; 
/// let taskid = TaskId::from(1); // ok
/// ```
/// Donot try to explictly construct `TaskId` from `0`.
/// Debug mode panic example (only compiles in debug):
/// ```should_panic
/// # use taskorch::cond::TaskId;
/// TaskId::from(0); // panics in debug
/// ```
///
/// Release mode behavior demonstration:
/// ```no_run
/// # use taskorch::task::TaskId;
/// let _ = TaskId::from(0); // would log warning in release
/// ```
impl From<usize> for TaskId {
    fn from(id: usize) -> Self {
        #[cfg(debug_assertions)]
        if id == 0 {
            panic!("TaskId cannot be zero");
        }
        #[cfg(not(debug_assertions))]
        if crate::log::LEVEL as usize >= crate::log::Level::Warn as usize && id == 0 {
            warn!("TaskId::new() the input id is zero, is not avaiable!");
        }
        Self::new(id)
    }
}

impl std::fmt::Debug for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(taskid) = self.0 {
            f.write_fmt(format_args!("TaskId({})",taskid.get()))
            // f.debug_tuple("TaskId").field(&v).finish()
        } else {
            f.write_fmt(format_args!("TaskId(None)"))
            // f.debug_tuple("TaskId").field(&v).finish()
        }
    }
}

// impl Deref for TaskId {
//     type Target = usize;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl DerefMut for TaskId {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// /// just for debug message
// #[doc(hidden)]
// #[repr(transparent)]
// pub(crate) struct TaskIdOption(pub(crate) Option<TaskId>);

// impl std::fmt::Debug for TaskIdOption {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if let Some(taskid) = self.0 {
//             f.write_fmt(format_args!("TaskId({})",taskid.as_usize()))
//             // f.debug_tuple("TaskId").field(&v).finish()
//         } else {
//             f.write_fmt(format_args!("TaskId(None)"))
//             // f.debug_tuple("TaskId").field(&v).finish()
//         }
//     }
// }

#[test]
fn test_tid() {
    let tid = TaskId::from(3);
    let tid: TaskId = 3.into();
    let tid = TaskId::new(0);
    // let tid = TaskIdOption(Some(tid));
    // println!("{tid:?}");
    // let tid = TaskIdOption(None);
    // println!("{tid:?}");
}

/// A 0-based index of representing the position of a parameter in fun or closure signature.
/// 
/// # Examples:
/// ```rust
/// fn ff(a:i32,b:i16,c:char) {}
/// 
/// let pi = Pi::P0; // the index postion of 1st `a:i32` (index 0)
/// let pi = Pi::P1; // the index postion of 2nd `b:i16` (index 1)
/// let pi = Pi::P2; // the index postion of 3rd `c:char` (index 2)
/// let pi = Pi::from(2); // equivalent Pi::P2
/// let pi = 2.into(); // equivalent Pi::P2
/// ```
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct ArgIdx<T>(pub(crate) u8,PhantomData<T>);
impl<T> ArgIdx<T> {
    pub const AI0:ArgIdx<T> = ArgIdx(0,PhantomData);
    pub const AI1:ArgIdx<T> = ArgIdx(1,PhantomData);
    pub const AI2:ArgIdx<T> = ArgIdx(2,PhantomData);
    pub const AI3:ArgIdx<T> = ArgIdx(3,PhantomData);
    pub const AI4:ArgIdx<T> = ArgIdx(4,PhantomData);
    pub const AI5:ArgIdx<T> = ArgIdx(5,PhantomData);
    pub const AI6:ArgIdx<T> = ArgIdx(6,PhantomData);
    pub const AI7:ArgIdx<T> = ArgIdx(7,PhantomData);
    pub const AI8:ArgIdx<T> = ArgIdx(8,PhantomData);
    const  AINONE:ArgIdx<T> = ArgIdx(u8::MAX,PhantomData);

    pub(crate) const fn const_new<const i:u8>() -> Self {
        ArgIdx(i,PhantomData)
    }
}
impl<T> ArgIdx<T> {
    pub(crate) const fn i(&self)->u8 {
        self.0
    }
}
impl<T> From<u8> for ArgIdx<T> {
    #[inline]
    fn from(pi: u8) -> Self {
        Self(pi,PhantomData)
    }
}
impl<T> From<ArgIdx<T>> for u8 {
    #[inline]
    fn from(pi: ArgIdx<T>) -> Self {
        pi.0
    }
}

impl<T> Debug for ArgIdx<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arg<{name}>({:?})", &self.0, name=type_name::<T>())
        // f.debug_tuple("Pi").field(&self.0).field(&type_name::<T>()).finish()
    }
}

/// added the position property for Arg
/// at input/output/inner.

#[derive(PartialEq,Debug)]
pub(crate) enum Place {
    Input,
    Output,
}

// impl<T> Debug for Place<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Input(arg0) =>
//                 f.write_fmt(format_args!("Input<>({})",arg0.i())),
//             Self::Output(arg0) =>
//                 f.write_fmt(format_args!("Output<>({})",arg0.i())),
//         }
//     }
// }

/// Cond Addr
/// Represents the position where a condition occurs — specifically, the position of a parameter.
///
/// This is determined by a combination of the task ID and zero-based condition index,
/// which together uniquely identify where the parameter is located in the system.
// #[derive(Clone, Copy)]
#[derive(PartialEq)]
pub struct CondAddr<T>{
    taskid: TaskId,
    place: Place,
    arg: ArgIdx<T>
}

impl<T> CondAddr<T> {
    pub const NONE: Self = Self {
        taskid:TaskId::NONE,
        place: Place::Input,
        arg: ArgIdx::AINONE,
    };
    pub(crate) const fn new<const i:u8>()->Self {
        Self {
            taskid: TaskId::NONE,
            place: Place::Input,
            arg: ArgIdx::const_new::<i>(),
        }
    }
}

impl<T> CondAddr<T> {
    #[inline]
    pub const fn taskid(&self)->TaskId {
        self.taskid
    }
    #[inline]
    pub const fn place(&self)->&Place {
        &self.place
    }
    #[inline]
    pub const fn argi(&self)->&ArgIdx<T> {
        &self.arg
    }
    #[inline]
    pub fn set(&mut self, id:TaskId, place: Place, i:ArgIdx<T>) {
        self.taskid = id;
        self.place = place;
        self.arg = i;
    }
    #[inline]
    pub fn set_taskid(&mut self, id:TaskId) {
        self.taskid = id
    }
}

impl<T> Default for CondAddr<T> {
    fn default() -> Self {
        Self::from((TaskId::NONE, Place::Input, ArgIdx::AINONE))
    }
}

impl<T> From<(TaskId,Place,ArgIdx<T>)> for CondAddr<T> {
    fn from((tid,place,argi): (TaskId,Place,ArgIdx<T>)) -> Self {
        Self {
            taskid: tid,
            place,
            arg: argi,
        }
    }
}

impl<T> Debug for CondAddr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"CA<{typename}>({:?},{}({:?}))",
            &self.taskid,
            if self.place == Place::Input {"Input"} else {"Output"},
            self.argi().i(),
            typename=type_name::<T>())
        // f.debug_tuple("CondAddr").field(&self.0).field(&self.1).finish()
    }
}
