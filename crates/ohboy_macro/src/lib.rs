use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input};

#[proc_macro]
pub fn byte_permutations(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);

    match process_byte_permutations(input) {
        Ok(t) => t,
        Err(e) => e.into_compile_error().into(),
    }
}

fn process_byte_permutations(input: syn::LitStr) -> Result<TokenStream, syn::Error> {
    let pattern = input.value();
    
    let pattern = pattern
        .strip_prefix("0b")
        .ok_or(syn::Error::new_spanned(&input, "expected pattern to start with 0b"))?;

    let mut permutations: Vec<String> = vec![String::new()];

    for bit in pattern.chars() {
        match bit {
            '0' | '1' =>  {
                for p in permutations.iter_mut() {
                    p.push(bit);
                }
            },
            'x' | 'X' => {
                permutations = permutations
                    .into_iter()
                    .flat_map(|p| [p.clone() + "0", p + "1"])
                    .collect();
            },
            '_' => {},
            // TODO: fix error message haha
            _ => return Err(syn::Error::new_spanned(input, "expected pattern to start with 0b"))
        }
    }

    let literals: Vec<syn::LitInt> = permutations
        .into_iter()
        .map(|p| syn::LitInt::new(&format!("0b{p}"), Span::call_site().into()))
        .collect();

    Ok(quote! { #(#literals)|* }.into())
}

struct MatchBitsArgs {
    value: syn::Ident,
    pattern: syn::LitStr
}

impl syn::parse::Parse for MatchBitsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let value: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let pattern: syn::LitStr = input.parse()?;

        Ok (MatchBitsArgs { value, pattern })
    }
}

#[proc_macro]
pub fn match_bits(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MatchBitsArgs);
    
    match process_match_bits(args.value, args.pattern) {
        Ok(t) => t,
        Err(e) => e.into_compile_error().into(),
    }
}

fn process_match_bits(value: syn::Ident, pattern: syn::LitStr) -> Result<TokenStream, syn::Error> {
    let pattern_val = pattern.value();
    
    let pattern_val = pattern_val
        .strip_prefix("0b")
        .ok_or(syn::Error::new_spanned(&pattern, "expected pattern to start with 0b"))?;

    let mut first_bit_pos: Option<usize> = None;
    let mut mask_length: usize = 0;

    for (i, bit) in pattern_val.chars().rev().enumerate() {
        match bit {
            '0' | '1' => { },
            'x' | 'X' => {
                mask_length += 1;
                if first_bit_pos.is_none() { first_bit_pos = Some(i) }
            },
            '_' => {},
            // TODO: fix error message haha
            _ => return Err(syn::Error::new_spanned(pattern, "expected pattern to start with 0b"))
        }
    }

    let mask = syn::LitInt::new(&format!("0b{}", "1".repeat(mask_length)), Span::call_site().into());
    let first_bit_pos = first_bit_pos.unwrap();

    Ok(quote!{ (#value >> #first_bit_pos) & #mask }.into())
}
