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
