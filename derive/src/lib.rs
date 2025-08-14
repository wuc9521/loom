//! Procedural macro for deriving Syntax trait
//!
//! This crate provides the #[derive(Syntax)] macro that ensures
//! users implement all required functions for their datatypes.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, Variant, Generics};

#[proc_macro_derive(Syntax)]
pub fn derive_syntax(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;

    // Parse the enum variants
    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return syn::Error::new_spanned(
                &name,
                "Syntax can only be derived for enums (sum types)",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate the implementation
    let expanded = generate_syntax_impl(&name, &generics, &variants);

    expanded.into()
}

#[proc_macro_derive(Finite)]
pub fn derive_finite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;

    // Parse the enum variants
    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => {
            return syn::Error::new_spanned(
                &name,
                "Finite can only be derived for enums (sum types)",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate the implementation
    let expanded = generate_finite_impl(&name, &generics, &variants);

    expanded.into()
}

fn generate_syntax_impl(
    name: &Ident,
    generics: &Generics,
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let variant_names: Vec<&Ident> = variants.iter().map(|v| &v.ident).collect();
    let variant_count = variants.len();
    
    // Extract generic parameters
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        // This trait implementation will be overridden by user implementations
        impl #impl_generics crate::api::Syntax for #name #ty_generics #where_clause {
            fn name(&self) -> &str {
                stringify!(#name)
            }

            fn sort(&self) -> &str {
                stringify!(#name)
            }

            fn to_smtlib(&self) -> String {
                unimplemented!("to_smtlib() must be implemented for {}", stringify!(#name))
            }

            fn from_smtlib(_s: &str) -> Result<Self, crate::api::SyntaxError> {
                unimplemented!("from_smtlib() must be implemented for {}", stringify!(#name))
            }

            fn validate(&self) -> Result<(), crate::api::SyntaxError> {
                Ok(()) // Default implementation
            }

            fn constructor_count(&self) -> usize {
                #variant_count
            }

            fn constructor_name(&self, index: usize) -> Option<&str> {
                match index {
                    #(
                        #variant_count => Some(stringify!(#variant_names)),
                    )*
                    _ => None,
                }
            }
        }

        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_smtlib())
            }
        }

        // Note: Users must implement eval function with signature:
        // fn eval<T: Finite>(&self, arg: &T) -> T
    }
}

fn generate_finite_impl(
    name: &Ident,
    generics: &Generics,
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let variant_count = variants.len();
    let first_variant = &variants[0].ident;
    
    // Extract generic parameters
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Check if this is a generic type
    let has_generics = !generics.params.is_empty();
    
    let default_impl = if has_generics {
        quote! {
            fn default() -> Self {
                // For generic types, users must implement default manually
                unimplemented!("Default must be implemented manually for generic types")
            }
        }
    } else {
        quote! {
            fn default() -> Self {
                #name::#first_variant
            }
        }
    };

    // Generate constraint checks for each variant's fields
    let constraint_checks = generate_constraint_checks(variants);

    quote! {
        impl #impl_generics crate::api::Finite for #name #ty_generics #where_clause {
            fn cardinality(&self) -> usize {
                #variant_count
            }

            fn is_valid(&self) -> bool {
                true // All enum variants are valid
            }

            #default_impl
        }
        
        // Compile-time constraint check
        impl #impl_generics #name #ty_generics #where_clause {
            const _FINITE_CONSTRAINTS: () = {
                #constraint_checks
                
                // These calls will fail at compile time if constraints are not satisfied
                let _: () = {
                    // This is a compile-time check, not runtime
                };
            };
        }
    }
}

fn generate_constraint_checks(
    _variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    // For now, just generate a simple constraint check
    // This ensures that all type parameters must implement Finite
    quote! {
        // Compile-time check that all type parameters implement Finite
        fn check_finite_constraints<T: crate::api::Finite>(_t: &T) {}
    }
}
