# Bounce

The uncomplicated state management library for Yew.

Bounce is inspired by [Redux](https://github.com/reduxjs/redux) and
[Recoil](https://github.com/facebookexperimental/Recoil).

## Example

For bounce states to function, a `<BounceRoot />` must be registered.

```
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

```
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
