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
    fn run(states: &BounceStates, input: Self::Input) -> LocalBoxFuture<'_, Self::Output>;
}

/// A deferred result type for future notions.
///
/// For each future notion `T`, a `Deferred<T>` the following notions will be applied to states:
///
/// - A `Deferred::<T>::Pending` Notion will be applied before a future notion starts running.
/// - A `Deferred::<T>::Complete` Notion will be applied after a future notion completes.
/// - If any states are used during the run of a future notion,
///   a `Deferred::<T>::Outdated` Notion will be applied **once** after the value of any used states changes.
#[derive(Debug)]
pub enum Deferred<T>
where
    T: FutureNotion,
{
    /// A future notion is running.
    Pending { input: T::Input },
    /// A future notion has completed.
    Completed { input: T::Input, output: T::Output },
    /// The states used in the future notion run has been changed.
    Outdated { input: T::Input },
}

impl<T> Deferred<T>
where
    T: FutureNotion,
{
    /// Returns `true` if current future notion is still running.
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending { .. } => true,
            Self::Completed { .. } => false,
            Self::Outdated { .. } => false,
        }
    }

    /// Returns `true` if current future notion has been completed.
    pub fn is_completed(&self) -> bool {
        match self {
            Self::Pending { .. } => false,
            Self::Completed { .. } => true,
            Self::Outdated { .. } => false,
        }
    }

    /// Returns `true` if current future notion is outdated.
    pub fn is_outdated(&self) -> bool {
        match self {
            Self::Pending { .. } => false,
            Self::Completed { .. } => false,
            Self::Outdated { .. } => true,
        }
    }

    /// Returns the input of current future notion.
    pub fn input(&self) -> T::Input {
        match self {
            Self::Pending { input } => (*input).clone_rc(),
            Self::Completed { input, .. } => (*input).clone_rc(),
            Self::Outdated { input } => (*input).clone_rc(),
        }
    }

    /// Returns the output of current future notion if it has completed.
    pub fn output(&self) -> Option<T::Output> {
        match self {
            Self::Pending { .. } => None,
            Self::Completed { output, .. } => Some((*output).clone_rc()),
            Self::Outdated { .. } => None,
        }
    }
}

impl<T> Clone for Deferred<T>
where
    T: FutureNotion,
{
    fn clone(&self) -> Self {
        match self {
            Self::Pending { ref input } => Self::Pending {
                input: input.clone_rc(),
            },
            Self::Completed {
                ref input,
                ref output,
            } => Self::Completed {
                input: input.clone_rc(),
                output: output.clone_rc(),
            },
            Self::Outdated { ref input } => Self::Outdated {
                input: input.clone_rc(),
            },
        }
    }
}
