# Bounce

The uncomplicated state management library for Yew.

Bounce is inspired by [Redux](https://github.com/reduxjs/redux) and
[Recoil](https://github.com/facebookexperimental/Recoil).


## Rationale

Currently, Yew Context API and Yewdux have the following limitations:

- Too much boilerplate.

   You either have to manually control whether to notify
   subscribers (Yewdux), or you have to manually define contexts (Context API).

   I wish changes can be detected automatically by `PartialEq` or
something similar and not have to define contexts manually.

- State change notifies all.

   State changes will notify all subscribers.
   `useSelector` hook for `Redux` will only trigger a re-render when the
selected value changes. However, currently both the Context API and
Yewdux will notify all subscribers when a change happens.

- Does not utilise contexts (Yewdux).

   This is somewhat my personal preference.
   I prefer easily resettable / overridable contexts over global state (Agent).

- Needless clones (Context).

   Context API will produce a clone of the state to all subscribers.

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
            <input type_="text" oninput={on_text_input} value={username.inner.to_string()} />
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

That's it!

You can find the full example [here](https://github.com/futursolo/bounce/tree/master/examples/simple).

## License

Bounce is dual licensed under the MIT license and the Apache License (Version 2.0).
