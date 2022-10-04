use std::collections::BTreeMap;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::root_state::BounceRootState;
use crate::states::slice::{use_slice_dispatch, use_slice_value};
use crate::utils::Id;
use crate::Slice;

pub(crate) enum ArtifactAction<T: PartialEq + 'static> {
    Insert(Id, Rc<T>),
    Remove(Id),
}

#[derive(PartialEq, Slice)]
pub(crate) struct ArtifactSlice<T>
where
    T: PartialEq + 'static,
{
    inner: BTreeMap<Id, Rc<T>>,
}

impl<T> Default for ArtifactSlice<T>
where
    T: PartialEq + 'static,
{
    fn default() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
}

impl<T: PartialEq + 'static> Clone for ArtifactSlice<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Reducible for ArtifactSlice<T>
where
    T: PartialEq + 'static,
{
    type Action = ArtifactAction<T>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut self_ = (*self).clone();

        match action {
            ArtifactAction::Insert(id, artifact) => self_.inner.insert(id, artifact),
            ArtifactAction::Remove(id) => self_.inner.remove(&id),
        };

        self_.into()
    }
}

impl<T> ArtifactSlice<T>
where
    T: PartialEq + 'static,
{
    pub(crate) fn get(&self) -> Vec<Rc<T>> {
        self.inner.values().cloned().collect()
    }
}

/// A hook to read all artifacts of the current artifact type.
///
/// An artifact is a global side effect (e.g.: document title) that will be collected in the
/// rendering order.
///
/// # Note
///
/// If you are trying to manipulate elements in the `<head />` element (e.g.: document title),
/// it is recommended to use the [Helmet](crate::helmet) API instead.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # use std::rc::Rc;
/// #
/// #[derive(Debug, PartialEq)]
/// pub struct Title {
///     inner: String,
/// }
///
/// #[function_component(Inner)]
/// fn inner() -> Html {
///     html! {<Artifact<Title> value={Rc::new(Title { inner: "My Title 2".into() })} />}
/// }
///
/// #[function_component(Outer)]
/// fn outer() -> Html {
///     html! {
///         <>
///             <Artifact<Title> value={Rc::new(Title { inner: "My Title 1".into() })} />
///             <Inner />
///         </>
///     }
/// }
///
/// # fn inner() {
/// // [Title { inner: "My Title 1" }, Title { inner: "My Title 2" }]
/// let title_artifacts = use_artifacts::<Title>();
/// # }
/// ```
#[hook]
pub fn use_artifacts<T>() -> Vec<Rc<T>>
where
    T: PartialEq + 'static,
{
    use_slice_value::<ArtifactSlice<T>>().get()
}

/// Properties of the [`Artifact`] Component.
#[derive(Debug, Properties, PartialEq, Eq)]
pub struct ArtifactProps<T>
where
    T: PartialEq + 'static,
{
    /// The Rc'ed value of the artifact.
    pub value: Rc<T>,
}

/// A component to register an artifact.
///
/// The artifact is registered in rendering order and is collected into a vector
/// that can be read with the [`use_artifacts`] hook.
///
/// # Note
///
/// If you are trying to manipulate elements in the `<head />` element (e.g.: document title),
/// it is recommended to use the [Helmet](crate::helmet) API instead.
///
/// # Example
///
/// ```
/// # use yew::prelude::*;
/// # use bounce::prelude::*;
/// # use std::rc::Rc;
/// #
/// #[derive(Debug, PartialEq)]
/// pub struct Title {
///     inner: String,
/// }
///
/// let artifact = Rc::new(Title { inner: "My Title".into() });
///
/// let rendered = html! {<Artifact<Title> value={artifact} />};
/// ```
#[function_component(Artifact)]
pub fn artifact<T>(props: &ArtifactProps<T>) -> Html
where
    T: PartialEq + 'static,
{
    let id = *use_state(Id::new);

    // we need to register root as a dependency of effects so that when the root changes the artifact can
    // be moved from 1 root to another.
    let root = use_context::<BounceRootState>().expect_throw("No bounce root found.");

    let artifact_dispatch = use_slice_dispatch::<ArtifactSlice<T>>();

    {
        let artifact_dispatch = artifact_dispatch.clone();
        use_effect_with_deps(
            move |(val, _)| {
                artifact_dispatch(ArtifactAction::Insert(id, val.clone()));
                || {}
            },
            (props.value.clone(), root.clone()),
        );
    }

    use_effect_with_deps(
        move |_| {
            move || {
                artifact_dispatch(ArtifactAction::Remove(id));
            }
        },
        root,
    );

    Html::default()
}
