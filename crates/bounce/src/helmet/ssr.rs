use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

// The static renderer can run outside of the Yew runtime.
// We use a send oneshot channel for this purpose.
use futures::channel::oneshot as sync_oneshot;

use crate::root_state::BounceStates;

use super::state::{merge_helmet_states, HelmetState};
use super::FormatTitle;

use yew::prelude::*;

pub struct StaticWriterInner {
    start_rx: sync_oneshot::Receiver<()>,
    tx: sync_oneshot::Sender<String>,
}

/// The writer of [StaticRenderer].
///
/// This writer is passed to a `<HelmetProvider />` for tags to be rendered with it.
#[derive(Clone)]
pub struct StaticWriter {
    inner: Rc<Cell<Option<StaticWriterInner>>>,
}

impl PartialEq for StaticWriter {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl fmt::Debug for StaticWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticWriter").field("inner", &"_").finish()
    }
}

impl StaticWriter {
    pub(crate) async fn send_helmet(
        &self,
        states: BounceStates,
        format_title: Option<FormatTitle>,
        default_title: Option<AttrValue>,
    ) {
        let StaticWriterInner { start_rx, tx } = match self.inner.take() {
            Some(m) => m,
            None => return,
        };

        // The StaticRenderer is dropped, we don't render anything.
        if start_rx.await.is_err() {
            return;
        }

        let helmet_states = states.get_artifacts::<HelmetState>();
        let tags = merge_helmet_states(&helmet_states, format_title.as_ref(), default_title);

        let mut s = String::new();
        for tag in tags {
            // Writing to String can never fail.
            let _ = tag.write_static(&mut s);
        }

        // We ignore cases where the StaticRenderer is dropped.
        let _ = tx.send(s);
    }
}

/// A Helmet Static Renderer.
///
/// This renderer provides support to statically render helmet tags to string to be prefixed to a
/// server-side rendered artifact.
#[derive(Debug)]
pub struct StaticRenderer {
    start_tx: sync_oneshot::Sender<()>,
    rx: sync_oneshot::Receiver<String>,
}

impl StaticRenderer {
    /// Creates a new Static Renderer - Static Writer pair.
    pub fn new() -> (StaticRenderer, StaticWriter) {
        let (start_tx, start_rx) = sync_oneshot::channel();
        let (tx, rx) = sync_oneshot::channel();

        (
            StaticRenderer { start_tx, rx },
            StaticWriter {
                inner: Rc::new(Cell::new(Some(StaticWriterInner { start_rx, tx }))),
            },
        )
    }

    /// Renders the helmet tags collected in the current renderer.
    ///
    /// # Notes
    ///
    /// For applications using streamed server-side rendering, the renderer will discard any tags
    /// rendered after this method is called.
    pub async fn render(self) -> String {
        self.start_tx.send(()).expect("failed to start rendering.");
        self.rx.await.expect("failed to receive value.")
    }
}
