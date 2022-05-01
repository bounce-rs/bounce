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

### Helmet Bridge

The `<HelmetBridge />` component is used to customise the behaviour and
responsible of reconciling the elements to the `<head />` element.

```rust
html! {
    <BounceRoot>
        <HelmetBridge default_title="default title" />
        // other components.
    </BounceRoot>
}
```

The Helmet Bridge component accepts two properties,
a `default_title` which will be applied when no other title elements
are registered and a `format_title` function which is used to format
the title before it is passed to the document.
