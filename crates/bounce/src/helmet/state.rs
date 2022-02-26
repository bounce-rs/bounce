use std::rc::Rc;

use gloo::utils::document;
use web_sys::Element;

#[derive(PartialEq, Debug)]
pub(crate) struct HelmetState {
    pub tags: Vec<Rc<HelmetTag>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum HelmetTag {
    Title(Rc<str>),
}

impl HelmetTag {
    pub fn apply(&self) -> Option<Element> {
        match self {
            Self::Title(m) => {
                document().set_title(m);

                None
            }
        }
    }

    pub fn detach(&self, element: Option<Element>) {
        if let Some(m) = element {
            m.parent_element().as_ref().map(|m| m.remove_child(m));
        }

        match self {
            Self::Title(_) => {}
        }
    }
}
