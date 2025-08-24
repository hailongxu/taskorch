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

    /// Construct a TaskId from usize
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
    /// let taskid = TaskId::new(1); // ok
    /// ```
    /// Donot try to explictly construct `TaskId` from `0`.
    /// Debug mode panic example (only compiles in debug):
    /// ```should_panic
    /// # use taskorch::cond::TaskId;
    /// TaskId::new(0); // panics in debug
    /// ```
    ///
    /// Release mode behavior demonstration:
    /// ```no_run
    /// # use taskorch::task::TaskId;
    /// let _ = TaskId::new(0); // would log warning in release
    /// ```
    #[inline]
    pub fn new(id:usize)->Self {
        #[cfg(debug_assertions)]
        if id == 0 {
            panic!("TaskId cannot be zero");
        }
        #[cfg(not(debug_assertions))]
        if crate::log::LEVEL as usize >= crate::log::Level::Warn as usize && id == 0 {
            warn!("TaskId::new() the input id is zero, is not avaiable!");
        }
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
impl From<usize> for TaskId {
    fn from(id: usize) -> Self {
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
pub struct Pi<T>(pub(crate) u8,PhantomData<T>);
impl<T> Pi<T> {
    pub const PI0:Pi<T> = Pi(0,PhantomData);
    pub const PI1:Pi<T> = Pi(1,PhantomData);
    pub const PI2:Pi<T> = Pi(2,PhantomData);
    pub const PI3:Pi<T> = Pi(3,PhantomData);
    pub const PI4:Pi<T> = Pi(4,PhantomData);
    pub const PI5:Pi<T> = Pi(5,PhantomData);
    pub const PI6:Pi<T> = Pi(6,PhantomData);
    pub const PI7:Pi<T> = Pi(7,PhantomData);
    pub const PI8:Pi<T> = Pi(8,PhantomData);
    const  PINONE:Pi<T> = Pi(u8::MAX,PhantomData);

    pub(crate) const fn const_new<const i:u8>() -> Self {
        Pi(i,PhantomData)
    }
}
impl<T> Pi<T> {
    const fn i(&self)->u8 {
        self.0
    }
}
impl<T> From<u8> for Pi<T> {
    #[inline]
    fn from(pi: u8) -> Self {
        Self(pi,PhantomData)
    }
}
impl<T> From<Pi<T>> for u8 {
    #[inline]
    fn from(pi: Pi<T>) -> Self {
        pi.0
    }
}

impl<T> Debug for Pi<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pi<{name}>({:?})", &self.0, name=type_name::<T>())
        // f.debug_tuple("Pi").field(&self.0).field(&type_name::<T>()).finish()
    }
}

/// Cond Addr
/// Represents the position where a condition occurs — specifically, the position of a parameter.
///
/// This is determined by a combination of the task ID and zero-based condition index,
/// which together uniquely identify where the parameter is located in the system.
// #[derive(Clone, Copy)]
#[derive(PartialEq)]
pub struct CondAddr<T>(TaskId,Pi<T>);
    // where Pi<T>: Copy;
impl<T> CondAddr<T> {
    pub const NONE: Self = Self(TaskId::NONE,Pi::PINONE);
    pub(crate) const fn const_new<const i:u8>()->Self {
        Self(TaskId::NONE, Pi::const_new::<i>())
    }
}
impl<T> CondAddr<T> {
    #[inline]
    pub const fn taskid(&self)->TaskId {
        self.0
    }
    #[inline]
    pub const fn pi(&self)->&Pi<T> {
        &self.1
    }
    #[inline]
    pub fn set(&mut self, id:TaskId, i:Pi<T>) {
        self.0 = id;
        self.1 = i;
    }
    #[inline]
    pub fn set_taskid(&mut self, id:TaskId) {
        self.0 = id
    }
}

impl<T> Default for CondAddr<T> {
    fn default() -> Self {
        Self(TaskId::NONE, Pi::PINONE)
    }
}

impl<T> From<(TaskId,Pi<T>)> for CondAddr<T> {
    fn from((tid,pi): (TaskId,Pi<T>)) -> Self {
        Self(tid, pi)
    }
}

impl<T> Debug for CondAddr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"CA<{name}>({:?},Pi({:?}))",&self.0,&self.1.i(),name=type_name::<T>())
        // f.debug_tuple("CondAddr").field(&self.0).field(&self.1).finish()
    }
}
