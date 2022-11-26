use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write;
use std::iter;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// The static renderer can run outside of the Yew runtime.
// We use a send oneshot channel for this purpose.
use futures::channel::oneshot as sync_oneshot;

use crate::root_state::BounceStates;

use super::state::{merge_helmet_states, HelmetState, HelmetTag};
use super::FormatTitle;

use yew::prelude::*;

pub struct StaticWriterInner {
    start_rx: sync_oneshot::Receiver<()>,
    tx: sync_oneshot::Sender<Vec<u8>>,
}

/// The writer of [StaticRenderer].
///
/// This writer is passed to a `<HelmetBridge />` for tags to be rendered with it.
#[derive(Clone)]
pub struct StaticWriter {
    inner: Arc<Mutex<Option<StaticWriterInner>>>,
}

impl PartialEq for StaticWriter {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for StaticWriter {}

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
        let StaticWriterInner { start_rx, tx } = match self.inner.lock().unwrap().take() {
            Some(m) => m,
            None => return,
        };

        // The StaticRenderer is dropped, we don't render anything.
        if start_rx.await.is_err() {
            return;
        }

        let helmet_states = states.get_artifacts::<HelmetState>();
        let tags = merge_helmet_states(&helmet_states, format_title.as_ref(), default_title);

        // We ignore cases where the StaticRenderer is dropped.
        let _ = tx.send(
            bincode::serialize(
                &tags
                    .into_iter()
                    .map(|m| Rc::try_unwrap(m).unwrap_or_else(|e| (*e).clone()))
                    .collect::<Vec<_>>(),
            )
            .expect("failed to serialize helmet tags"),
        );
    }
}

/// A Helmet Static Renderer.
///
/// This renderer provides support to statically render helmet tags to string to be prefixed to a
/// server-side rendered artifact.
#[derive(Debug)]
pub struct StaticRenderer {
    start_tx: sync_oneshot::Sender<()>,
    rx: sync_oneshot::Receiver<Vec<u8>>,
}

impl StaticRenderer {
    /// Creates a new Static Renderer - Static Writer pair.
    pub fn new() -> (StaticRenderer, StaticWriter) {
        let (start_tx, start_rx) = sync_oneshot::channel();
        let (tx, rx) = sync_oneshot::channel();

        (
            StaticRenderer { start_tx, rx },
            StaticWriter {
                inner: Arc::new(Mutex::new(Some(StaticWriterInner { start_rx, tx }))),
            },
        )
    }

    /// Renders the helmet tags collected in the current renderer.
    ///
    /// # Notes
    ///
    /// For applications using streamed server-side rendering, the renderer will discard any tags
    /// rendered after this method is called.
    pub async fn render(self) -> Vec<HelmetTag> {
        self.start_tx.send(()).expect("failed to start rendering.");
        let helmet_buf = self.rx.await.expect("failed to receive value.");

        bincode::deserialize(&helmet_buf).expect("failed to deserialize helmet tags")
    }
}

impl HelmetTag {
    fn write_attrs_from(
        w: &mut dyn Write,
        attrs: &BTreeMap<Rc<str>, Rc<str>>,
        write_data_attr: bool,
    ) -> fmt::Result {
        let mut data_tag_written = false;

        for (index, (name, value)) in attrs
            .iter()
            .map(|(name, value)| (&**name, &**value))
            .chain(iter::from_fn(|| {
                (write_data_attr && !data_tag_written).then(|| {
                    data_tag_written = true;
                    ("data-bounce-helmet", "pre-render")
                })
            }))
            .enumerate()
        {
            if index > 0 {
                write!(w, " ")?;
            }

            write!(
                w,
                r#"{}="{}""#,
                name,
                html_escape::decode_script_double_quoted_text(value)
            )?;
        }

        Ok(())
    }

    /// Writes the attributes of the current tag into a `std::fmt::Write`.
    ///
    /// You can use this method to write attributes to the `<html></html>` or `<body></body>` tag.
    pub fn write_attrs(&self, w: &mut dyn Write) -> fmt::Result {
        match self {
            Self::Title(_) => Ok(()),
            Self::Body { attrs } | Self::Html { attrs } => Self::write_attrs_from(w, attrs, false),
            Self::Meta { attrs }
            | Self::Link { attrs }
            | Self::Script { attrs, .. }
            | Self::Style { attrs, .. }
            | Self::Base { attrs } => Self::write_attrs_from(w, attrs, true),
        }
    }

    /// Writes the content of a tag into a `std::fmt::Write`.
    ///
    /// `<html ...>` and `<body ...>` tags are not written.
    ///
    /// To write attributes for html and body tags,
    /// you can use the [`write_attrs`](Self::write_attrs) method instead.
    pub fn write_static(&self, w: &mut dyn Write) -> fmt::Result {
        match self {
            Self::Title(m) => {
                write!(w, "<title>{}</title>", m)
            }
            Self::Script { content, attrs, .. } => {
                write!(w, "<script ")?;
                Self::write_attrs_from(w, attrs, true)?;
                write!(w, ">{}</script>", content)
            }
            Self::Style { content, attrs } => {
                write!(w, "<style ")?;
                Self::write_attrs_from(w, attrs, true)?;
                write!(w, ">{}</style>", content)
            }
            Self::Body { .. } => Ok(()),
            Self::Html { .. } => Ok(()),
            Self::Base { attrs } => {
                write!(w, "<base ")?;
                Self::write_attrs_from(w, attrs, true)?;
                write!(w, ">")
            }
            Self::Link { attrs } => {
                write!(w, "<link ")?;
                Self::write_attrs_from(w, attrs, true)?;
                write!(w, ">")
            }
            Self::Meta { attrs } => {
                write!(w, "<meta ")?;
                Self::write_attrs_from(w, attrs, true)?;
                write!(w, ">")
            }
        }
    }
}
