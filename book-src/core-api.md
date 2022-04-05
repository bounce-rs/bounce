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

A derived state. Unlike Atoms or Slices, a selector cannot store any
value. It derives its value from other states (atoms, slices or
selectors) and subscribes to the state used to derive values
automatically so it will update its value whenever any value it
subscribes to changes.

### Input Selector

A derived state family. Similar to Selectors, but also allows an
additional input to be provided to select states.

### Notion

A action that can be applied to multiple states.

### Future Notion

A future notion is a notion that is applied twice upon the initiation
and completion of an asynchronous task.

### Artifact

An artifact is a side-effect API that collects all values of a state
registered in its defining order.

### Observer

The observer API can be used to create an observed state that notifies
the observer when a state changes.
