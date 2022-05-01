# Core API

The core API contains a set of APIs that is enabled by default. It is
built with a hook and function component first approach. The value of a
state is shared among all components under a bounce root.

### Atom

A simple state that is initialised with its default value and
allows a new value to be set by a setter. This state is similar to a
state defined by `use_state_eq` hook but its default value is determined
by `Default::default()` and its value is shared by all components under
a bounce root.

Atoms are defined by deriving the Atom macro.
Any type that implements `Default` + `PartialEq` can `#[derive(Atom)]`.

```rust
#[derive(Atom, PartialEq, Default)]
pub struct Counter {
    inner: u32
}
```

When the first time its used by a hook, it will be initialised by it's
default value.

```rust
// The counter will be initialised with its initial value when first
// called.
let ctr = use_atom::<Counter>();

let increment = {
    let ctr = ctr.clone();
    Callback::from(move || {
        // A new value of type `Counter` can be set with `.set`
        ctr.set(Counter { inner: ctr.inner + 1 });
    })
};

let decrement = {
    let ctr = ctr.clone();
    Callback::from(move || {
        // A new value of type `Counter` can be set with `.set`
        ctr.set(Counter { inner: ctr.inner - 1 });
    })
};

html! {
    // The handle of `use_atom` implements `Deref` to `Counter`.
    <div>{ctr.inner}</div>
    <button onclick={increment}>{"+"}</button>
    <button onclick={decrement}>{"-"}</button>
}
```

`use_atom` can be use to create a bidirectional connection between a
component and an atom. You may also `use_atom_value` or `use_atom_setter`
to create a read-only or write-only connection.

**API Reference:**

- [`use_atom`](https://docs.rs/bounce/0.2.0/bounce/fn.use_atom.html)
- [`use_atom_value`](https://docs.rs/bounce/0.2.0/bounce/fn.use_atom_value.html)
- [`use_atom_setter`](https://docs.rs/bounce/0.2.0/bounce/fn.use_atom_setter.html)
- [`#[derive(Atom)]`](https://docs.rs/bounce/0.2.0/bounce/derive.Atom.html)


### Slice

Similar to Atoms, but allows a `use_reducer_eq`-like usage where actions
can be dispatched with actions to mutate the state. This allows an
action to be applied on a complex state.
In addition to `Default` and `PartialEq`, Slices also need to implement
the `Reducible` trait from Yew.

The counter example rewritten as a Slice:

```rust
use yew::prelude::*;

pub enum CounterAction {
    Increment,
    Decrement,
}

#[derive(Slice, PartialEq, Default)]
pub struct Counter {
    inner: u32
}

impl Reducible for Counter {
    type Action = CounterAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CounterAction::Increment => {
                Self { inner: self.inner + 1 }.into()
            }
            CounterAction::Decrement => {
                Self { inner: self.inner - 1 }.into()
            }
        }
    }
}
```

Instead of setting a new value, slices receive actions as updates and a
new value is created by the `reduce` function. Slices in Bounce is
Copy-on-Write that you only need to clone the value if it needs to be
mutated.

Actions can be dispatched by using the `dispatch()` method on the the
`use_slice` handle.

```rust
// The counter will be initialised with its initial value when first
// called.
let ctr = use_slice::<Counter>();

let increment = {
    let ctr = ctr.clone();
    Callback::from(move || {
        // An Action can be dispatched by the `.dispatch()` method.
        ctr.dispatch(CounterAction::Increment);
    })
};

let decrement = {
    let ctr = ctr.clone();
    Callback::from(move || {
        ctr.dispatch(CounterAction::Decrement);
    })
};

html! {
    // The handle of `use_atom` implements `Deref` to `Counter`.
    <div>{ctr.inner}</div>
    <button onclick={increment}>{"+"}</button>
    <button onclick={decrement}>{"-"}</button>
}
```

**API Reference:**

- [`use_slice`](https://docs.rs/bounce/0.2.0/bounce/fn.use_slice.html)
- [`use_slice_value`](https://docs.rs/bounce/0.2.0/bounce/fn.use_slice_value.html)
- [`use_slice_dispatch`](https://docs.rs/bounce/0.2.0/bounce/fn.use_slice_dispatch.html)
- [`#[derive(Slice)]`](https://docs.rs/bounce/0.2.0/bounce/derive.Slice.html)

### Selector

A derived state. Unlike Atoms or Slices, a selector does not store any
value in itself. It derives its value from other states (atoms, slices or
other selectors) and subscribes to the state used to derive values
so it will update its value when any value it
subscribes to changes automatically.

#### Example

A selector that checks if the counter slice defined in the previous
example is even.

```rust
#[derive(PartialEq)]
pub struct IsEven {
    inner: bool,
}

impl Selector for IsEven {
    fn select(states: &BounceStates) -> Rc<Self> {
        // The IsEven selector will subscribe to the Counter slice.
        // If the value of the counter slice changes,
        // the value of IsEven will be updated as well.
        let val = states.get_slice_value::<Counter>();

        Self {
            inner: val.inner % 2 == 0,
        }
        .into()
    }
}
```

API Reference:

- [`Selector`](https://docs.rs/bounce/0.2.0/bounce/trait.Selector.html)
- [`use_selector_value`](https://docs.rs/bounce/0.2.0/bounce/fn.use_selector_value.html)

### Input Selector

A derived state family. Similar to Selectors, but also allows an
additional input to be provided to select states.

An input selector will refresh its value upon either the input or the selected states change.

API Reference:

- [`InputSelector`](https://docs.rs/bounce/0.2.0/bounce/trait.InputSelector.html)
- [`use_input_selector_value`](https://docs.rs/bounce/0.2.0/bounce/fn.use_input_selector_value.html)

### Notion

An action that can be applied to multiple states.

When a notion is applied, it will be broadcasted to all states that
listen to this notion.

To listen to a notion, apply `#[with_notion(NotionType)]` tag to your
slice or atom and define how it can be applied with the
`WithNotion<NotionType>` trait.

```
use yew::prelude::*;

pub struct Reset;

pub enum CounterAction {
    Increment,
    Decrement,
}

#[derive(Slice, PartialEq, Default)]
#[with_notion(Reset)] // The slice that listens to a notion of type T needs to be denoted with `#[with_notion(T)]`.
pub struct Counter {
    inner: u32
}

impl Reducible for Counter {
    type Action = CounterAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CounterAction::Increment => {
                Self { inner: self.inner + 1 }.into()
            }
            CounterAction::Decrement => {
                Self { inner: self.inner - 1 }.into()
            }
        }
    }
}

// A WithNotion<T> is required for each notion denoted in the #[with_notion] attribute.
impl WithNotion<Reset> for Counter {
    fn apply(self: Rc<Self>, _notion: Rc<Reset>) -> Rc<Self> {
        Self::default().into()
    }
}
```

A notion can be applied with the `use_notion_applier` hook.

```
let reset_everything = use_notion_applier::<Reset>();
reset_everything(Reset);
```

API Reference:

- [`WithNotion`](https://docs.rs/bounce/0.2.0/bounce/trait.WithNotion.html)
- [`use_notion_applier`](https://docs.rs/bounce/0.2.0/bounce/fn.use_notion_applier.html)

### Future Notion

A notion that is applied upon the initiation and completion of an asynchronous task.

A future notion can be defined with the `#[future_notion]` attribute.

```rust
struct User {
    id: u64,
    name: String,
}

#[future_notion(FetchUser)]
async fn fetch_user(id: &u64) -> User {
    // fetch user

    User { id: *id, name: "John Smith".into() }
}
```

Slices and Atoms can receive updates of a future notion by listening to
the Deferred notion.

```rust
#[derive(PartialEq, Default, Atom)]
#[with_notion(Deferred<FetchUser>)]  // A future notion with type `T` will be applied as `Deferred<T>`.
struct UserState {
    inner: Option<Rc<User>>,
}

// Each time a future notion is run, it will be applied twice.
impl WithNotion<Deferred<FetchUser>> for UserState {
    fn apply(self: Rc<Self>, notion: Rc<Deferred<FetchUser>>) -> Rc<Self> {
        match notion.output() {
            Some(m) => Self { inner: Some(m) }.into(),
            None => self,
        }
    }
}
```

Future Notions can be initiated with the `use_future_notion_runner`
hook.

```rust
let load_user = use_future_notion_runner::<FetchUser>();
load_user(1);
```

API Reference:

- [`#[future_notion]`](https://docs.rs/bounce/0.2.0/bounce/attr.future_notion.html)
- [`Deferred`](https://docs.rs/bounce/0.2.0/bounce/enum.Deferred.html)
- [`use_future_notion_runner`](https://docs.rs/bounce/0.2.0/bounce/fn.use_future_notion_runner.html)

#### Note

The Future Notion API is a low-level API to execute asynchronous tasks.

If you want to interact with an API,
it is recommended to use the [Query API](query-api.md) instead.

The Query API is built with the Future Notion API.

### Artifact

An artifact is a side-effect API that collects all values of a state
registered in its defining order.

This API is useful when declaring global side effects (e.g.: document title).

<!-- API Reference:                                                                 -->

<!-- - [`Artifact`](https://docs.rs/bounce/0.2.0/bounce/type.Artifact.html)         -->
<!-- - [`use_artifacts`](https://docs.rs/bounce/0.2.0/bounce/fn.use_artifacts.html) -->

#### Note

If you are trying to manipulate elements in the `<head />` element (e.g.: document title),
it is recommended to use the [Helmet API](helmet-api.md) instead.

### Observer

The observer API can be used to create an observed state that notifies
the observer when a state changes.

This API can be used to persist a state to the local storage or
synchronise it to other tabs.
