use anymap2::AnyMap;
use yew::prelude::*;

use crate::root_state::BounceRootState;

/// Properties for [`BounceRoot`].
#[derive(Properties, Debug, PartialEq, Clone)]
pub struct BounceRootProps {
    /// Children of a Bounce Root.
    #[prop_or_default]
    pub children: Children,

    /// A callback that retrieves an `AnyMap` that contains initial states.
    ///
    /// States not provided will use `Default`.
    ///
    /// This only affects [`Atom`](macro@crate::Atom) and [`Slice`](macro@crate::Slice).
    #[prop_or_default]
    pub get_init_states: Option<Callback<(), AnyMap>>,
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
    let BounceRootProps {
        children,
        get_init_states,
    } = props.clone();

    let root_state = (*use_state(move || {
        let init_states = get_init_states.map(|m| m.emit(())).unwrap_or_default();
        BounceRootState::new(init_states)
    }))
    .clone();

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

    #[allow(clippy::unused_unit, clippy::redundant_clone)]
    {
        let _root_state = root_state.clone();
        let _ = use_transitive_state!(
            move |_| -> () {
                #[cfg(feature = "ssr")]
                #[cfg(feature = "helmet")]
                {
                    // Workaround to send helmet states back to static writer
                    use crate::helmet::StaticWriterState;

                    let states = _root_state.states();
                    let writer_state = states.get_atom_value::<StaticWriterState>();

                    if let Some(ref w) = writer_state.writer {
                        w.send_helmet(
                            states,
                            writer_state.format_title.clone(),
                            writer_state.default_title.clone(),
                        );
                    }
                }

                // We drop the root state on SSR as well.
                _root_state.clear();
            },
            ()
        );
    }

    html! {
        <ContextProvider<BounceRootState> context={root_state}>{children}</ContextProvider<BounceRootState>>
    }
}
