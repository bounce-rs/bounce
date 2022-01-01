use std::any::Any;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use once_cell::sync::Lazy;

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Id(u64);

impl Default for Id {
    fn default() -> Self {
        static CTR: Lazy<AtomicU64> = Lazy::new(AtomicU64::default);

        Self(CTR.fetch_add(1, Ordering::SeqCst))
    }
}

impl Id {
    pub fn new() -> Self {
        Self::default()
    }
}

pub(crate) struct Listener {
    _listener: Rc<dyn Any>,
}

impl Listener {
    pub fn new(inner: Rc<dyn Any>) -> Self {
        Self { _listener: inner }
    }
}

pub trait RcTrait {
    type Inner: 'static;

    fn clone_rc(&self) -> Self;
}

impl<T> RcTrait for Rc<T>
where
    T: 'static,
{
    type Inner = T;

    fn clone_rc(&self) -> Rc<Self::Inner> {
        self.clone()
    }
}
