# derive2

Provides `#[derive2(...)]`, an alternative to `#[derive(...)]` that allows the use of macro_rules macros.

# Usage

`derive2` takes all its arguments, and calls them like function macros with the item it is applied to:

```rust
#[derive2(macro0, macro1)]
#[derive(Default)]
pub enum Foo {
    #[default]
    Bar,
    #[macro0(something)]
    Baz,
}
```

expands to

```rust
#[derive(Default)]
pub enum Foo {
    #[default]
    Bar,
    Baz,
}

macro0! {
    #[derive(Default)]
    pub enum Foo {
        #[default]
        Bar,
        #[macro0(something)]
        Baz,
    }
}
macro1! {
    #[derive(Default)]
    pub enum Foo {
        #[default]
        Bar,
        #[macro0(something)]
        Baz,
    }
}
```
