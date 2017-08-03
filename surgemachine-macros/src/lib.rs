extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(IndexedEnum)]
pub fn choose_enum(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_choose_enum(&ast);
    gen.parse().unwrap()
}

fn impl_choose_enum(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    if let syn::Body::Enum(ref body) = ast.body {
        let num_items = body.len() as u64;
        let (from_index_cases, to_index_cases): (Vec<_>, Vec<_>) = body.iter().enumerate()
            .map(|(i, case)| {
                let unqualified_ident = &case.ident;
                let ident = quote! { #name::#unqualified_ident };
                let index = i as u64;

                let index_to_ident = match case.data {
                    syn::VariantData::Unit => quote! { #index => #ident },
                    _ => panic!("Unsupported enum case {} in IndexedEnum", ident),
                };

                let ident_to_index = match case.data {
                    syn::VariantData::Unit => quote! { #ident => #index },
                    _ => panic!("Unsupported enum case {} in IndexedEnum", ident),
                };

                (index_to_ident, ident_to_index)
            }).unzip();

        quote! {
            impl IndexedEnum for #name {
                const NUM_ITEMS: u64 = #num_items;
                fn to_index (&self) -> u64 {
                    match *self {
                        #(#to_index_cases),*,
                    }
                }
                fn from_index(index: u64) -> Self {
                    match index {
                        #(#from_index_cases),*,
                        _ => panic!("Invalid index {} for {}", index, stringify!(#name))
                    }
                }
            }
        }
    } else {
       panic!("#[derive(IndexedEnum)] is only defined for enums!");
    }
}
#[test]
fn test_choose_enum () {
    let input = quote! {
        #[derive(IndexedEnum)]
        enum TestEnum {
            One,
            Two,
        }
    };

    let expected = quote! {
        impl IndexedEnum for TestEnum {
            const NUM_ITEMS: u64 = 2u64;
            fn to_index (&self) -> u64 {
                match *self {
                    TestEnum::One => 0u64,
                    TestEnum::Two => 1u64,
                }
            }

            fn from_index(index: u64) -> Self {
                match index {
                    0u64 => TestEnum::One,
                    1u64 => TestEnum::Two,
                    _ => panic!("Invalid index {} for {}", index, stringify!(TestEnum))
                }
            }
        }
    };

    let input_str = input.to_string();
    let ast = syn::parse_derive_input(&input_str).unwrap();

    assert_eq!(
        impl_choose_enum(&ast).to_string(),
        expected.to_string()
    );
}
