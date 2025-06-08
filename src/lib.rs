extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

const MAX_LAYERS: usize = 4;
const QUADS_AMOUNT: usize = 4;

macro_rules! error {
    ($input:expr, $msg:expr) => {
        syn::Error::new_spanned($input, $msg)
            .to_compile_error()
            .into()
    };
}

fn ordinal(mut n: usize) -> String {
    n = n + 1;
    let suffix = match n {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{}{}", n, suffix)
}

fn get_sub_shape(input: &LitStr, sub_shape: &u8, error_postfix: &str) -> proc_macro2::TokenStream {
    // Ensure the sub-shape is valid
    match sub_shape {
        b'C' => quote! { Subshape::Circle },
        b'S' => quote! { Subshape::Square },
        b'R' => quote! { Subshape::Rectangle },
        b'W' => quote! { Subshape::Windmill },
        _ => error!(
            input,
            format!(
                "Invalid sub-shape \"{}\" in {}",
                *sub_shape as char, error_postfix
            )
        ),
    }
}

fn get_color(input: &LitStr, color: &u8, error_postfix: &str) -> proc_macro2::TokenStream {
    // Ensure the color is valid
    match color {
        b'r' => quote! { Color::Red },
        b'g' => quote! { Color::Green },
        b'b' => quote! { Color::Blue },
        b'y' => quote! { Color::Yellow },
        b'p' => quote! { Color::Purple },
        b'c' => quote! { Color::Cyan },
        b'w' => quote! { Color::White },
        b'u' => quote! { Color::Uncolored },
        _ => error!(
            input,
            format!("Invalid color \"{}\" in {}", *color as char, error_postfix)
        ),
    }
}

fn check_quad(
    input: &LitStr,
    quad: &[u8],
    layer_index: usize,
    quad_index: usize,
) -> Option<proc_macro2::TokenStream> {
    // Ensure the sub-shape and color are valid
    let sub_shape = quad[0];
    let color = quad[1];

    // Check for "--"
    if sub_shape == b'-' && color == b'-' {
        return None;
    }

    // Prepare the error postfix to be used in the error message
    let error_postfix = format!(
        "{} layer, {} quad",
        ordinal(layer_index),
        ordinal(quad_index)
    );

    // Check for the sub-shape
    let sub_shape_token = get_sub_shape(&input, &sub_shape, &error_postfix);
    let color_token = get_color(&input, &color, &error_postfix);

    Some(quote! { Some(Quad(#sub_shape_token, #color_token)) })
}

fn check_layer(input: &LitStr, layer: &str, layer_index: usize) -> proc_macro2::TokenStream {
    // Ensure the layer is valid
    if layer.len() != QUADS_AMOUNT * 2 {
        return if layer.len() % 2 == 0 {
            let more_or_less = if layer.len() > QUADS_AMOUNT * 2 {
                "more"
            } else {
                "less"
            };

            error!(
                input,
                format!(
                    "{} layer has {} than {} characters",
                    ordinal(layer_index),
                    more_or_less,
                    QUADS_AMOUNT * 2
                )
            )
        } else {
            error!(
                input,
                format!(
                    "{} layer has odd number of characters",
                    ordinal(layer_index),
                )
            )
        };
    }

    // Check every quad
    let mut none_count = 0;
    let mut quad_tokens = Vec::with_capacity(4);
    let quads = layer.as_bytes().chunks(2).collect::<Vec<&[u8]>>();
    for (quad_index, &quad) in quads.iter().enumerate() {
        match check_quad(input, quad, layer_index, quad_index) {
            Some(quad_token) => quad_tokens.push(quad_token),
            None => {
                quad_tokens.push(quote! { None });
                none_count += 1
            }
        }
    }

    if none_count == QUADS_AMOUNT {
        return error!(input, format!("{} layer is empty", ordinal(layer_index)));
    }

    quote! { [ #(#quad_tokens),* ] }
}

fn check_key(input: &LitStr, shape: &str) -> proc_macro2::TokenStream {
    // Ensure the layer count is valid
    let layers = shape.split(':').collect::<Vec<&str>>();
    if layers.len() > MAX_LAYERS {
        return error!(input, format!("Input has more than {} layers", MAX_LAYERS));
    }

    let mut layer_tokens = vec![];
    for (layer_index, &layer) in layers.iter().enumerate() {
        let layer_token = check_layer(input, layer, layer_index);
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
/// - An empty key is passed
/// - The key contains more than 4 layers
/// - A layer contains more or less than 4 quads
/// - A quad contains invalid sub-shape or color
/// - An empty layer is passed
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
    if short_key.is_empty() {
        return error!(input, "Empty input");
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
