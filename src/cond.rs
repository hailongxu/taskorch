//! # `cond` module
//!
//! ## Concept of Conditions
//!
//! - Each task has multiple **inputs** (called **conditions**)
//! - Each function has multiple **inputs** (called **parameters**)
//! - A task can be described by a function, where its conditions correspond to the function's parameters
//!
//! ## Locating a Condition
//!
//! The [`CondAddr<T>`] type serves as a **logical address** that uniquely identifies a condition's location and type.
//!
//! It contains four components:
//!
//! 1. **`taskid`**: Identifies which task the condition belongs to (see: [`TaskId`])
//! 2. **`cond_index`**: The index of the condition within its task (see: [`ArgIdx`])
//! 3. **`section`**: Identifies the interface section (input or output) for this parameter (see: [`Section`])
//! 4. **`Type`**: The built-in data type of the condition (expressed through the generic parameter `T`)
//!
//! ## How These Components Work Together
//!
//! - Components **1-3** form a **logical address** that specifies **where** to find the condition
//! - Component **4** provides **type annotation** that specifies **what type of data** to expect
//!
//! ## Important Note
//!
//! This is purely a **descriptive address** - it identifies where data is located and what type it has,
//! but does not itself provide access to the data. Actual data access requires separate mechanisms
//! and is not implemented via [`Deref`] or similar traits on this type.
//!
//! This combination provides a complete, type-safe way to identify task conditions.



use std::{any::type_name, marker::PhantomData, num::NonZeroUsize, fmt::Debug};

/// Logical address identifying the location and type of a condition within the system.
///
/// Represents the position of an input parameter (condition) in a task's interface, uniquely identified by:
/// - [`TaskId`]: Which task contains the condition
/// - [`ArgIdx<T>`]: The zero-based index and type of the parameter
/// - [`Section`]: **Currently reserved for internal use** (distinguishes between input and output contexts)
///
/// The generic parameter `T` provides type safety by specifying the expected data type
/// of the condition value at this address.
///
/// # Current Usage
/// In the current API, users only work with **input conditions**. The `Section` component
/// is used internally by the system and is not exposed in the public interface.
///
/// # Note  
/// This is a descriptive logical address, not a memory address. It identifies where
/// data is located and what type it has, but does not itself provide data access.

#[derive(PartialEq)]
pub struct CondAddr<T>{
    taskid: TaskId,
    section: Section,
    argidx: ArgIdx<T>
}

impl<T> CondAddr<T> {
    /// The `NONE` address does not point to any condition.
    pub const NONE: Self = Self {
        taskid: TaskId::NONE,
        section: Section::Input,
        argidx: ArgIdx::AINONE,
    };
    pub(crate) const fn new<const I:u8>()->Self {
        Self {
            taskid: TaskId::NONE,
            section: Section::Input,
            argidx: ArgIdx::const_new::<I>(),
        }
    }
}

impl<T> CondAddr<T> {
    #[inline]
    pub const fn taskid(&self)->TaskId {
        self.taskid
    }
    #[inline]
    pub const fn section(&self)->&Section {
        &self.section
    }
    #[inline]
    pub const fn argidx(&self)->&ArgIdx<T> {
        &self.argidx
    }
    #[inline]
    pub fn set(&mut self, id:TaskId, section: Section, i:ArgIdx<T>) {
        self.taskid = id;
        self.section = section;
        self.argidx = i;
    }
    #[inline]
    pub fn set_taskid(&mut self, id:TaskId) {
        self.taskid = id
    }
}

impl<T> Default for CondAddr<T> {
    fn default() -> Self {
        Self::from((TaskId::NONE, Section::Input, ArgIdx::AINONE))
    }
}

/// 
/// construct a CondAddr
/// Args:
/// - #1: TaskId
/// - #2: Section (Input or Output)
/// - #3: ArgIdx (argument index, based from 0)
/// Returns:
/// - return CondAddr<T>
/// 
/// ## Exmaples:
/// ```rust
/// # use taskorch::{CondAddr,TaskId,ArgIdx,Section};
/// // cond addr is at (Task#1 and Task.Param#0) 
/// let ca = CondAddr::<i32>::from((TaskId::from(1),Section::Input,ArgIdx::AI0)); 
/// ```
impl<T> From<(TaskId,Section,ArgIdx<T>)> for CondAddr<T> {
    fn from((tid,section,argi): (TaskId,Section,ArgIdx<T>)) -> Self {
        Self {
            taskid: tid,
            section,
            argidx: argi,
        }
    }
}

impl<T> Debug for CondAddr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"CondAddr<{typename}>{{{:?},{}({:?})}}",
            &self.taskid,
            if self.section == Section::Input {"Input"} else {"Output"},
            self.argidx().i(),
            typename=type_name::<T>())
        // f.debug_tuple("CondAddr").field(&self.0).field(&self.1).finish()
    }
}


/// A unique identifier for a task within a given pool instance system.
///
/// # Zero/Non-Zero Semantics
///
/// - **Zero `TaskId`**: Reserved for internal use only (auto-assigned for **tasks without conditions**)
/// - **Caller-created IDs**: Attempts to create zero IDs will log warnings and are discouraged
/// - **Explicit non-zero IDs**: Required for all **tasks with conditions**
///
/// # Conversion
///
/// You can convert a `TaskId` to `usize` using the [`as_usize()`](TaskId::as_usize) method.
/// 
/// # Example:
/// ```
/// # use taskorch::TaskId;
/// let taskid = TaskId::from(3);
/// let taskid: TaskId = 3.into();
/// let uid = taskid.as_usize();
/// ```
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct TaskId(pub(crate) Option<NonZeroUsize>);

impl TaskId {
    /// A special [`TaskId`] value representing the absence of a task.
    ///
    /// # Purpose
    /// - Serves as a dummy task ID for **tasks without conditions** that don't require a real identifier
    /// - Reserved for internal system use
    ///
    /// # Usage Notes
    /// - Use this constant directly via `TaskId::NONE`
    /// - **Do not** attempt to create equivalent values using `TaskId::from(0)` or similar constructors
    ///   - In `Debug` mode: this will panic or result in undefined behavior
    ///   - In `Release` mode: this will generate warnings and may lead to system inconsistencies
    pub const NONE: Self = Self(None);

    /// Construct a TaskId from usize，just use in self crate, not expose it.
    /// and does not output any message.
    #[inline]
    pub(crate) const fn new(id:usize)->Self {
        Self(NonZeroUsize::new(id))
    }

    /// convert the `TaskId` to `usize`
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
/// # use taskorch::TaskId; 
/// let taskid = TaskId::from(1); // ok
/// ```
/// Donot try to explictly construct `TaskId` from `0`.
/// Debug mode panic
/// 
/// example (only compiles in debug):
/// ```should_panic
/// # use taskorch::TaskId;
/// TaskId::from(0); // panics in debug
/// ```
///
/// Release mode behavior demonstration:
/// ```no_run
/// # use taskorch::TaskId;
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

/// A zero-based index representing the position of a parameter in a function or closure signature.
///
/// The generic parameter `T` indicates the type of the parameter at this index position.
/// This type `T` is propagated to the corresponding [`CondAddr<T>`], ensuring consistent
/// type annotation throughout the condition addressing system.
/// 
/// # Examples:
/// ```rust
/// # use taskorch::ArgIdx;
/// 
/// fn ff(a:i32,b:i16,c:char) {}
/// 
/// // with `input``
/// let _ai = ArgIdx::<i8>::AI0; // point to a:i32
/// let _ai = ArgIdx::<i8>::from(2); // point to c:char
/// let _ai: ArgIdx::<i8> = 2.into(); // point to c:char
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
    pub const AINONE:ArgIdx<T> = ArgIdx(u8::MAX,PhantomData);

    pub(crate) const fn const_new<const I:u8>() -> Self {
        ArgIdx(I,PhantomData)
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

#[test]
fn test_argidx() {
    let _ai = ArgIdx::<i8>::AI0; // equivalent Pi::P2
    let _ai = ArgIdx::<i8>::from(2); // equivalent Pi::P2
    let _ai: ArgIdx::<i8> = 2.into(); // equivalent Pi::P2
}

/// Specifies the position context of an argument within a task's interface.
///
/// This is currently used internally by the system to distinguish between
/// different argument contexts. User-facing APIs currently only expose
/// input arguments.
///
/// # Variants
/// - `Input`: Argument provided as input to the task
/// - `Output`: Argument produced as output from the task (internal use only)
#[derive(PartialEq, Debug)]
pub enum Section {
    /// Argument provided as input to the task
    Input,
    /// Argument produced as output from the task (reserved for internal system use)
    Output,
}

// impl<T> Debug for Section<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Input(arg0) =>
//                 f.write_fmt(format_args!("Input<>({})",arg0.i())),
//             Self::Output(arg0) =>
//                 f.write_fmt(format_args!("Output<>({})",arg0.i())),
//         }
//     }
// }

