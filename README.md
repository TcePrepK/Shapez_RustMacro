# shapez_shape_macro

A Rust procedural macro to construct `Shape` structures for the game [shapez](https://shapez.io)
from short-form shape keys like `"RuCw--Cw:----Ru--"`.

## Usage

```rust
use shapez_macro::shapez_shape;

let shape = shapez_shape!("RuCw--Cw:----Ru--");
```

## Features

- Validates the input shape key
- Compile-time error messages

