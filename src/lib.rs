extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

const MAX_LAYERS: usize = 4;
const QUADS_AMOUNT: usize = 4;
const SUB_SHAPES: [u8; 4] = [b'C', b'S', b'R', b'W'];
const COLORS: [u8; 8] = [b'r', b'g', b'b', b'y', b'p', b'c', b'w', b'u'];

macro_rules! error {
    ($input:expr, $msg:expr) => {
        syn::Error::new_spanned($input, $msg)
            .to_compile_error()
            .into()
    };
}

fn get_sub_shape(input: &LitStr, sub_shape: &u8) -> proc_macro2::TokenStream {
    // Ensure the sub-shape is valid
    if !SUB_SHAPES.contains(sub_shape) {
        return error!(input, "Invalid sub-shape");
    }

    match sub_shape {
        b'C' => quote! { Subshape::Circle },
        b'S' => quote! { Subshape::Square },
        b'R' => quote! { Subshape::Rectangle },
        b'W' => quote! { Subshape::Windmill },
        _ => unreachable!(),
    }
}

fn get_color(input: &LitStr, color: &u8) -> proc_macro2::TokenStream {
    // Ensure the color is valid
    if !COLORS.contains(color) {
        return error!(input, "Invalid color");
    }

    match color {
        b'r' => quote! { Color::Red },
        b'g' => quote! { Color::Green },
        b'b' => quote! { Color::Blue },
        b'y' => quote! { Color::Yellow },
        b'p' => quote! { Color::Purple },
        b'c' => quote! { Color::Cyan },
        b'w' => quote! { Color::White },
        b'u' => quote! { Color::Uncolored },
        _ => unreachable!(),
    }
}

fn check_quad(input: &LitStr, quad: &[u8]) -> Option<proc_macro2::TokenStream> {
    // Ensure the sub-shape and color are valid
    let sub_shape = quad[0];
    let color = quad[1];

    // Check for "--"
    if sub_shape == b'-' && color == b'-' {
        return None;
    }

    // Check for the sub-shape
    let sub_shape_token = get_sub_shape(&input, &sub_shape);
    let color_token = get_color(&input, &color);

    Some(quote! { Some(Quad(#sub_shape_token, #color_token)) })
}

fn check_layer(input: &LitStr, layer: &str) -> proc_macro2::TokenStream {
    // Ensure the layer is valid
    if layer.len() != QUADS_AMOUNT * 2 {
        return error!(input, "Invalid layer");
    }

    // Ensure the quad amount is valid
    let quads = layer.as_bytes().chunks(2).collect::<Vec<&[u8]>>();
    if quads.len() != QUADS_AMOUNT {
        return error!(input, "Invalid quad amount");
    }

    let mut none_count = 0;
    let mut quad_tokens = Vec::with_capacity(4);
    for &quad in quads.iter() {
        match check_quad(input, quad) {
            Some(quad_token) => quad_tokens.push(quad_token),
            None => {
                quad_tokens.push(quote! { None });
                none_count += 1
            }
        }
    }

    if none_count == QUADS_AMOUNT {
        return error!(input, "Empty layer");
    }

    quote! { [ #(#quad_tokens),* ] }
}

fn check_key(input: &LitStr, shape: &str) -> proc_macro2::TokenStream {
    // Ensure the layer count is valid
    let layers = shape.split(':').collect::<Vec<&str>>();
    if layers.len() > MAX_LAYERS {
        return error!(input, "Invalid layer count");
    }

    let mut layer_tokens = vec![];
    for &layer in layers.iter() {
        let layer_token = check_layer(input, layer);
        layer_tokens.push(layer_token);
    }

    quote! { vec![ #(#layer_tokens),* ] }
}

/// Procedural macro to construct a `Shape` structure from a short-form shape key,
/// following the format used in the game [shapez](https://shapez.io).
///
/// # Syntax
///
/// ```
/// shapez_shape!("RuCw--Cw:----Ru--");
/// ```
///
/// Each pair of characters represents a 'Quad',
/// collecting a sub-shape (C, S, R, or W) and a color (r, g, b, y, p, c, w, or u).
/// A quad can be empty as well by using '-' for both characters.
/// Up to 4 layers can be defined, separated by colons ':'.
///
/// # Example
///
/// ```
/// use shapez_macro::shapez_shape;
///
/// let shape = shapez_shape!("RuCrSgWw:Rr------");
/// ```
///
/// This expands to:
///
/// ```ignore
/// Shape {
///     layers: vec![
///         [
///             Some(Quad(Subshape::Rectangle, Color::Uncolored)),
///             Some(Quad(Subshape::Circle, Color::Red)),
///             Some(Quad(Subshape::Square, Color::Green)),
///             Some(Quad(Subshape::Windmill, Color::White)),
///         ],
///         [
///             Some(Quad(Subshape::Rectangle, Color::Red)),
///             None,
///             None,
///             None,
///         ],
///     ],
/// }
/// ```
///
/// # Errors
///
/// Compile-time errors are emitted if:
/// - The key is shorter than 8 characters
/// - More than 4 layers are provided
/// - A layer has more or less than 4 quads
/// - An invalid sub-shape or color character is used
/// - An empty layer is provided
///
/// # Notes
/// - Valid characters for sub-shapes are 'C', 'S', 'R', and 'W'
/// - Valid characters for colors are 'r', 'g', 'b', 'y', 'p', 'c', 'w', and 'u'
///
/// # See Also
/// - [shapez](https://shapez.io)
/// - [shapez viewer](https://viewer.shapez.io/)
#[proc_macro]
pub fn shapez_shape(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);

    // Ensure the input is valid
    let short_key = input.value();
    if short_key.len() < 8 {
        return error!(input, "Invalid input");
    }

    // Layer by layer constructs the shape from the short key
    let shape_tokens = check_key(&input, &short_key);

    quote! {
        Shape {
            layers: #shape_tokens,
        }
    }
    .into()
}
