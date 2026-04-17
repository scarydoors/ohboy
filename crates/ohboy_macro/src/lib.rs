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
        .ok_or_else(|| syn::Error::new_spanned(&input, "expected pattern to start with 0b"))?;

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
            c => return Err(syn::Error::new_spanned(&pattern, format!("invalid character '{}' in binary literal", c)))
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
        .ok_or_else(|| syn::Error::new_spanned(&pattern, "expected pattern to start with 0b"))?;

    let (shift, mask_len) = pattern_val
        .chars()
        .rev()
        .filter(|c| !matches!(c, '_'))
        .enumerate()
        .try_fold((None::<usize>, 0usize), |(shift, mask_len), (i, c)| {
            match c {
                '0' | '1' => Ok((shift, mask_len)),
                'x' | 'X' => {
                    Ok((shift.or(Some(i)), mask_len + 1))
                },
                c => Err(syn::Error::new_spanned(&pattern, format!("invalid character '{}' in binary literal", c)))
            }
        })?;

    let mask = syn::LitInt::new(&format!("0b{}", "1".repeat(mask_len)), Span::call_site().into());
    // TODO: stop unwrapping here
    let shift = shift.unwrap();

    Ok(quote!{ (#value >> #shift) & #mask }.into())
}
