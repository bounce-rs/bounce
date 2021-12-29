use std::rc::Rc;

/// A trait to apply a notion on a state.
///
/// See: [`use_notion_applier`](crate::use_notion_applier)
pub trait WithNotion<T: 'static> {
    /// Applies a notion on current state.
    ///
    /// This always yields a new instance of [`Rc<Self>`] so it can be compared with the previous
    /// state using [`PartialEq`].
    fn apply(self: Rc<Self>, notion: Rc<T>) -> Rc<Self>;
}

// type BounceStates = ();
// type SuspensionResult<T> = std::result::Result<T, ()>;

// /// A notion that is suspendible while being processed.
// pub trait FutureNotion: Sized {
//     type Input: 'static;
//     type Output: 'static;

//     /// Creates a FutureNotion based on input.
//     /// Returns `None` if an input should be blocked
//     /// (e.g.: block execution when the same future notion is
//     // already running).
//     fn create(states: &BounceStates, input: Rc<Self::Input>) -> Option<Self>;

//     /// Attempts to resolve a future notion.
//     fn run(self: Rc<Self>) -> SuspensionResult<Self::Output>;
// }

// pub enum Futured<T>
// where
//     T: FutureNotion,
// {
//     Pending {
//         input: Rc<T::Input>,
//     },
//     Solved {
//         input: Rc<T::Input>,
//         output: Rc<T::Output>,
//     },
// }

// impl<T> Futured<T>
// where
//     T: FutureNotion,
// {
//     pub fn is_pending(&self) -> bool {
//         match self {
//             Self::Pending { .. } => true,
//             Self::Solved { .. } => false,
//         }
//     }
//
//     pub fn is_solved(&self) -> bool {
//         match self {
//             Self::Pending { .. } => false,
//             Self::Solved { .. } => true,
//         }
//     }

//     pub fn input(&self) -> Rc<T::Input> {
//         match self {
//             Self::Pending { input } => input.clone(),
//             Self::Solved { input, .. } => input.clone(),
//         }
//     }

//     pub fn output(&self) -> Option<Rc<T::Output>> {
//         match self {
//             Self::Pending { .. } => None,
//             Self::Solved { output, .. } => Some(output.clone()),
//         }
//     }
// }

// impl WithNotion<Futured<NotionA>> for SliceA {
//     fn apply(self: Rc<Self>, notion: Futured<NotionA>) -> Rc<Self> {
//         if let Some(m) = notion.output() {
//             ...
//         }
//     }
// }
