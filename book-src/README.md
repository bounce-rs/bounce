# Introduction

<p style="text-align: center; font-size: 2rem;">Bounce</p>

<p style="text-align: center;">
  <a href="https://crates.io/crates/bounce">
    <img src="https://img.shields.io/crates/v/bounce" alt="crates.io">
  </a>
  |
  <a href="https://docs.rs/bounce/">
    <img src="https://docs.rs/bounce/badge.svg" alt="docs.rs">
  </a>
  |
  <a href="https://github.com/futursolo/bounce">
    <img src="https://img.shields.io/github/stars/futursolo/bounce?style=social" alt="GitHub">
  </a>
</p>

Bounce is a state-management library focusing on simplicity and
performance.

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

## Installation

You can add it to your project with the following command:

```shell
cargo add bounce
```

## Getting Started

If you want to learn more about Bounce, you can check out the
[tutorial](./tutorial.md) and the [API documentation](https://docs.rs/bounce/).

## Licence

Bounce is dual licensed under the MIT license and the Apache License (Version 2.0).
