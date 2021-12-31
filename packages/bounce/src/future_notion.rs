use futures::future::LocalBoxFuture;
use std::rc::Rc;

use crate::root_state::BounceStates;

pub trait FutureNotion {
    type Input: 'static;
    type Output: 'static;

    fn run(
        states: BounceStates,
        input: Rc<Self::Input>,
    ) -> LocalBoxFuture<'static, Rc<Self::Output>>;
}

#[derive(Debug, Clone)]
pub enum Deferred<T>
where
    T: FutureNotion,
{
    Pending {
        input: Rc<T::Input>,
    },
    Complete {
        input: Rc<T::Input>,
        output: Rc<T::Output>,
    },
}

impl<T> Deferred<T>
where
    T: FutureNotion,
{
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending { .. } => true,
            Self::Complete { .. } => false,
        }
    }

    pub fn is_completed(&self) -> bool {
        match self {
            Self::Pending { .. } => false,
            Self::Complete { .. } => true,
        }
    }

    pub fn input(&self) -> Rc<T::Input> {
        match self {
            Self::Pending { input } => input.clone(),
            Self::Complete { input, .. } => input.clone(),
        }
    }
    pub fn output(&self) -> Option<Rc<T::Output>> {
        match self {
            Self::Pending { .. } => None,
            Self::Complete { output, .. } => Some(output.clone()),
        }
    }
}
