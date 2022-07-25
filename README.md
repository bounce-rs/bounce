# Bounce

[![Run Tests & Publishing](https://github.com/bounce-rs/bounce/actions/workflows/everything.yml/badge.svg)](https://github.com/bounce-rs/bounce/actions/workflows/everything.yml)
[![crates.io](https://img.shields.io/crates/v/bounce)](https://crates.io/crates/bounce)
[![docs.rs](https://docs.rs/bounce/badge.svg)](https://docs.rs/bounce/)

The uncomplicated state management library for Yew.

Bounce is inspired by [Redux](https://github.com/reduxjs/redux) and
[Recoil](https://github.com/facebookexperimental/Recoil).

## Rationale

Yew state management solutions that are currently available all have
some (or all) of the following limitations:

- Too much boilerplate.

   Users either have to manually control whether to notify
   subscribers or have to manually define contexts.

- State change notifies all.

   State changes will notify all subscribers.

- Needless clones.

   A clone of the state will be produced for all subscribers whenever
there's a change.

Bounce wants to be a state management library that:

- Has minimal boilerplate.

   Changes are automatically detected via `PartialEq`.

- Only notifies relevant subscribers.

   When a state changes, only hooks that subscribe to that state will
be notified.

- Reduces Cloning.

   States are `Rc`'ed.

## Example

For bounce states to function, a `<BounceRoot />` must be registered.

```rust
#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            {children}
        </BounceRoot>
    }
}
```

A simple state is called an `Atom`.

You can derive `Atom` for any struct that implements `PartialEq` and `Default`.

```rust
#[derive(PartialEq, Atom)]
struct Username {
    inner: String,
}

impl Default for Username {
    fn default() -> Self {
        Self {
            inner: "Jane Doe".into(),
        }
    }
}
```

You can then use it with the `use_atom` hook.

When an `Atom` is first used, it will be initialised with its `Default`
value.

```rust
#[function_component(Setter)]
fn setter() -> Html {
    let username = use_atom::<Username>();

    let on_text_input = {
        let username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            username.set(Username { inner: input.value().into() });
        })
    };

    html! {
        <div>
            <input type="text" oninput={on_text_input} value={username.inner.to_string()} />
        </div>
    }
}
```

If you wish to create a read-only (or set-only) handle, you can use
`use_atom_value` (or `use_atom_setter`).

```rust
#[function_component(Reader)]
fn reader() -> Html {
    let username = use_atom_value::<Username>();

    html! { <div>{"Hello, "}{&username.inner}</div> }
}
```

You can find the full example [here](https://github.com/futursolo/bounce/blob/master/examples/simple/src/main.rs).

## License

Bounce is dual licensed under the MIT license and the Apache License (Version 2.0).
