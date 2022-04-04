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

```
#[derive(Atom, PartialEq, Default)]
pub struct Counter {
    inner: u32
}
```

When the first time its used by a hook, it will be initialised by it's
default value.

```
let ctr = use_atom::<Counter>();
```

### Slice

Similar to Atoms, but allows a `use_reducer_eq`-like usage where actions
can be dispatched with actions to mutate the state. In addition
to `Default` and `PartialEq`, Slices also need to implement the `Reducible`
trait from Yew.

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
