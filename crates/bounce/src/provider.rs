use yew::prelude::*;

use crate::root_state::BounceRootState;

/// Properties for [`BounceRoot`].
#[derive(Properties, Debug, PartialEq)]
pub struct BounceRootProps {
    /// Children of a Bounce Root.
    #[prop_or_default]
    pub children: Children,
}

/// A `<BounceRoot />`.
///
/// For bounce states to function, A `<BounceRoot />` must present and registered as a context
/// provider.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # use bounce::BounceRoot;
/// #[function_component(App)]
/// fn app() -> Html {
///     html! {
///         <BounceRoot>
///             // children...
///         </BounceRoot>
///     }
/// }
///
/// ```
#[function_component(BounceRoot)]
pub fn bounce_root(props: &BounceRootProps) -> Html {
    let children = props.children.clone();

    let root_state = (*use_state(BounceRootState::new)).clone();

    {
        let root_state = root_state.clone();
        use_effect_with_deps(
            move |_| {
                // We clear all states manually.
                move || {
                    root_state.clear();
                }
            },
            (),
        );
    }

    // We drop the root state on SSR as well.
    #[allow(clippy::unused_unit)]
    {
        let _root_state = root_state.clone();
        let _ = use_transitive_state!(move |_| -> () { _root_state.clear() }, ());
    }

    html! {
        <ContextProvider<BounceRootState> context={root_state}>{children}</ContextProvider<BounceRootState>>
    }
}
