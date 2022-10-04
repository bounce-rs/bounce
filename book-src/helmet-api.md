# Helmet API

The Helmet API is an API to manipulate elements resided in the `<head />` element.

Elements can be applied with the `Helmet` component.

```rust
html! {
    <Helmet>
        // The title of current page.
        <title>{"page a title"}</title>
    </Helmet>
}
```

The Helmet API supports the following elements:

- title
- style
- script
- base
- link
- meta

The Helmet API supports setting attributes of the following elements:

- html
- body

### Helmet Provider

The `<HelmetProvider />` component is used to customise the behaviour and
responsible of reconciling the elements to the `<head />` element.

This should be used like a context provider. Helmet tags outside of the context
provider may not be rendered properly.

```rust
html! {
    <BounceRoot>
        <HelmetProvider default_title="default title">
            // other components.
        </HelmetProvider>
    </BounceRoot>
}
```

The Helmet Provider component accepts two properties,
a `default_title` which will be applied when no other title elements
are registered and a `format_title` function which is used to format
the title before it is passed to the document.

### API Reference:

- [`Helmet API`](https://docs.rs/bounce/0.3.0/bounce/helmet/index.html)
