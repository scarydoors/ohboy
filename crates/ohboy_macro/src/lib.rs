use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input};

#[proc_macro]
pub fn byte_pattern(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);

    match process_byte_pattern(input) {
        Ok(t) => t,
        Err(e) => e.into_compile_error().into(),
    }
}

fn process_byte_pattern(input: syn::LitStr) -> Result<TokenStream, syn::Error> {
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
