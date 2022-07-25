use std::rc::Rc;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::root_state::BounceRootState;

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

/// A hook to create a function that applies a `Notion`.
///
/// A `Notion` is an action that can be dispatched to any state that accepts the dispatched notion.
///
/// Any type that is `'static` can be dispatched as a notion.
///
/// Returns `Rc<dyn Fn(T)>`.
///
/// # Note
///
/// When states receives a notion, it will be wrapped in an `Rc<T>`.
///
/// # Example
///
/// ```
/// # use bounce::prelude::*;
/// # use std::fmt;
/// # use std::rc::Rc;
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// pub struct Reset;
///
/// #[derive(PartialEq, Atom)]
/// #[bounce(with_notion(Reset))] // A #[bounce(with_notion(Notion))] needs to be denoted for the notion.
/// struct Username {
///     inner: String,
/// }
///
/// // A WithNotion<T> is required for each notion denoted in the #[bounce(with_notion)] attribute.
/// impl WithNotion<Reset> for Username {
///     fn apply(self: Rc<Self>, _notion: Rc<Reset>) -> Rc<Self> {
///         Self::default().into()
///     }
/// }
///
/// // second state
/// #[derive(PartialEq, Atom, Default)]
/// #[bounce(with_notion(Reset))]
/// struct Session {
///     token: Option<String>,
/// }
///
/// impl WithNotion<Reset> for Session {
///     fn apply(self: Rc<Self>, _notion: Rc<Reset>) -> Rc<Self> {
///         Self::default().into()
///     }
/// }
/// #
/// # impl Default for Username {
/// #     fn default() -> Self {
/// #         Self {
/// #             inner: "Jane Doe".into(),
/// #         }
/// #     }
/// # }
/// #
/// # #[function_component(Setter)]
/// # fn setter() -> Html {
/// let reset_everything = use_notion_applier::<Reset>();
/// reset_everything(Reset);
/// # Html::default()
/// # }
/// ```
#[hook]
pub fn use_notion_applier<T>() -> Rc<dyn Fn(T)>
where
    T: 'static,
{
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    // Recreate the dispatch function in case root has changed.
    Rc::new(move |notion: T| {
        root.apply_notion(Rc::new(notion));
    })
}
