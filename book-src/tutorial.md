# Tutorial

This tutorial will guide you through to create a simple application that
greets the user with the entered name.

## 0. Prerequisites

This tutorial assumes the reader is familiar with the basics of Yew and
Rust. If you are new to Yew or Rust, you may find the following content
helpful:

1. [The Rust Programming Language](https://doc.rust-lang.org/book/)
2. [The `wasm-bindgen` Guide](https://rustwasm.github.io/wasm-bindgen/introduction.html)
3. [The Yew Docs](https://yew.rs/docs/getting-started/introduction)

You need the following tools:

1. [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target toolchain

   If you installed Rust with rustup, You can obtain the
`wasm32-unknown-unknown` target with the following command:

   ```shell
   rustup target add wasm32-unknown-unknown
   ```

2. The [Trunk](https://trunkrs.dev/#getting-started) Bundler

3. [`cargo-edit`](https://github.com/killercup/cargo-edit)

## 1. Prepare Dependencies

This tutorial uses the `yew-trunk-minimal-template`.
This template repository has a minimal setup of Yew with Trunk.
You can a new repository using the template with the following commands:

```shell
mkdir my-first-bounce-app
cd my-first-bounce-app
git init
git fetch --depth=1 -n https://github.com/yewstack/yew-trunk-minimal-template.git
git reset --hard $(git commit-tree FETCH_HEAD^{tree} -m "initial commit")
```

To add bounce to the dependencies, run the following command:

```shell
cargo add bounce
```

You can now view the app with Trunk:

```shell
trunk serve --open
```

## 2. Register Bounce Root

BounceRoot is a context provider that provides state management and
synchronisation mechanism to its child components.

It should be registered as a parent of all components that interact with
the bounce states. You only need 1 BounceRoot per application.

In this example, we can add it to the `App` component.

```rust
use bounce::BounceRoot;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        // Register BounceRoot so that states can be provided to its
        // children.
        <BounceRoot>
            <main>
                <img class="logo" src="https://yew.rs/img/logo.png" alt="Yew logo" />
                <h1>{ "Hello World!" }</h1>
                <span class="subtitle">{ "from Yew with " }<i class="heart" /></span>
            </main>
        </BounceRoot>
    }
}
```

## 3. Create an Atom

An atom is a simple state that is similar to a state created by
the `use_state` hook in Yew.

Atoms are created by deriving the Atom macro on a type. `Atom` can be derived
on any type that implements `Default` and `PartialEq`.

```rust
#[derive(Atom, PartialEq)]
pub struct Username {
    value: String,
}

impl Default for Username {
    fn default() -> Self {
        Username {
            value: "Jane Doe".into(),
        }
    }
}
```

## 4. Use an Atom

An Atom can be used in any component under a `BounceRoot` with the
following hooks:

- `use_atom`: Returns a `UseAtomHandle<T>`,
  which can be dereferenced to `T` and includes a `.set` method that can be
  used to update the value of this atom and triggers a re-render when
  the value changes.

- `use_atom_value`: Returns `Rc<T>` and triggers a re-render when
  the value changes.

- `use_atom_setter`: Returns a setter of type `Rc<dyn Fn(T)>` that can be used
  to update the value of `T`. This type will not trigger a re-render
  when the value changes.

## 5. Create a component that displays the username

To create a component that reads the value of an Atom, you can use the
`use_atom_value` hook mentioned in the previous chapter.

When the first time a state is used with a hook,
its value will be initialised with the value returned by
`Default::default()`.

```rust
#[function_component(Reader)]
fn reader() -> Html {
    let username = use_atom_value::<Username>();

    html! { <div>{"Hello, "}{&username.value}</div> }
}
```

## 6. Create a component to update the username

```rust
#[function_component(Setter)]
fn setter() -> Html {
    let username = use_atom::<Username>();

    let on_text_input = {
        let username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            username.set(Username {
                value: input.value(),
            });
        })
    };

    html! {
        <div>
            <input type="text" oninput={on_text_input} value={username.value.to_string()} />
        </div>
    }
}
```
