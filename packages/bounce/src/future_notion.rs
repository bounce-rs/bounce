use futures::future::LocalBoxFuture;

use crate::root_state::BounceStates;
use crate::utils::RcTrait;

/// A trait to implement a [`Future`](std::future::Future)-backed notion.
pub trait FutureNotion {
    /// The input type.
    type Input: RcTrait + 'static;
    /// The output type.
    type Output: RcTrait + 'static;

    /// Runs a future notion.
    fn run(states: BounceStates, input: Self::Input) -> LocalBoxFuture<'static, Self::Output>;
}

/// A deferred result type for future notions.
///
/// For each future notion `T`, a `Deferred<T>` will be applied to states twice.
///
/// A `Deferred::<T>::Pending` Notion will be applied before a future notion starts running and
/// a `Deferred::<T>::Complete` notion will be applied after a future notion completes.
#[derive(Debug, Clone)]
pub enum Deferred<T>
where
    T: FutureNotion,
{
    /// A future notion is running.
    Pending { input: T::Input },
    /// A future notion has completed.
    Complete { input: T::Input, output: T::Output },
}

impl<T> Deferred<T>
where
    T: FutureNotion,
{
    /// Returns `true` if current future notion is still running.
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending { .. } => true,
            Self::Complete { .. } => false,
        }
    }

    /// Returns `true` if current future notion has been completed.
    pub fn is_completed(&self) -> bool {
        match self {
            Self::Pending { .. } => false,
            Self::Complete { .. } => true,
        }
    }

    /// Returns the input of current future notion.
    pub fn input(&self) -> T::Input {
        match self {
            Self::Pending { input } => (*input).clone_rc(),
            Self::Complete { input, .. } => (*input).clone_rc(),
        }
    }

    /// Returns the output of current future notion if it has completed.
    pub fn output(&self) -> Option<T::Output> {
        match self {
            Self::Pending { .. } => None,
            Self::Complete { output, .. } => Some((*output).clone_rc()),
        }
    }
}
