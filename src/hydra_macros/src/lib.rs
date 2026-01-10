use std::{fmt::Debug, str::FromStr};

use funty::Integral;
use proc_macro::*;
use quote::{ToTokens, quote};
use syn::{ExprStruct, Fields, FieldsNamed, ItemStruct, LitInt, Stmt, Type, TypePath, TypeTuple, parse_macro_input, parse_quote, parse_str};

/// Attribute for structs which can be deserialized from an 8-bit register.
#[proc_macro_derive(DeserializedRegister8, attributes(width))]
pub fn deserialized_register_u8(item: TokenStream) -> TokenStream {
    deserialized_register_helper::<u8>(item)
}

/// Attribute for structs which can be deserialized from a 16-bit register.
#[proc_macro_derive(DeserializedRegister16, attributes(width))]
pub fn deserialized_register_u16(item: TokenStream) -> TokenStream {
    deserialized_register_helper::<u16>(item)
}

/// Attribute for structs which can be deserialized from a 32-bit register.
#[proc_macro_derive(DeserializedRegister32, attributes(width))]
pub fn deserialized_register_u32(item: TokenStream) -> TokenStream {
    deserialized_register_helper::<u32>(item)
}

/// Attribute for structs which can be deserialized from a 64-bit register.
#[proc_macro_derive(DeserializedRegister64, attributes(width))]
pub fn deserialized_register_u64(item: TokenStream) -> TokenStream {
    deserialized_register_helper::<u64>(item)
}

fn deserialized_register_helper<T>(item: TokenStream) -> TokenStream where T: Integral + ToTokens, <T as FromStr>::Err: Debug {
    // Parse input struct
    let item_struct = parse_macro_input!(item as ItemStruct);
    let item_struct_ident = item_struct.ident;
    let Fields::Named(FieldsNamed { named: fields, .. }) = &item_struct.fields else {
        return TokenStream::from(quote! {compile_error!()})
    };

    // Initialize variables to hold output syntax
    let mut output_stmts: Vec<Stmt> = Vec::new();
    let mut output_struct: ExprStruct = parse_quote! {
        #item_struct_ident {
            /* To be filled in below */
        }
    };

    // Iterate over struct fields in reverse
    let mut cumulative_width: T = T::ZERO;
    for field in fields.iter().rev() {
        // Parse #[width] attribute 
        let width = field.attrs.iter()
            .find(|attr| attr.path().is_ident("width"))
            .expect(format!("Field \"{}\" missing #[width(N)] attribute", field.ident.to_token_stream().to_string()).as_str())
            .parse_args::<LitInt>()
            .expect("Invalid argument provided to #[width(N)] attribute")
            .base10_digits()
            .parse::<T>()
            .expect("msg");

        // Add statements to From<T> implementation
        let field_ident = field.ident.as_ref().unwrap();
        match &field.ty {
            // () 
            Type::Tuple(tuple) if tuple.elems.is_empty() => {
                output_stmts.push(parse_quote! {
                    let #field_ident = ();
                });
            }
            // bool or int
            Type::Path(type_path) => {
                // Calculate & extract necessary variables, then append
                let bitmask = (T::ONE << width) - T::ONE;

                let optional_condition = if type_path.path.is_ident("bool") {quote!(!= 0)} else {TokenStream::new().into()};
                output_stmts.push(parse_quote! {
                    let #field_ident = ((value >> #cumulative_width) & #bitmask) #optional_condition;
                });
            }
            _ => {
                // TODO: error handling
            }
        }

        // All fields must be represented within the struct literal
        output_struct.fields.push(
            parse_quote!(#field_ident)
        );
        
        cumulative_width += width;
    }

    // Abort if width of struct does not encompass all bits
    if cumulative_width.as_u32() != T::BITS {
        panic!("Total bit width must be {}", T::BITS)
    }

    // Final output
    let source_type = parse_str::<Type>(std::any::type_name::<T>()).unwrap();
    let i = TokenStream::from(quote! {
        impl From<#source_type> for #item_struct_ident {
            fn from(value: #source_type) -> Self {
                #(#output_stmts)*
                #output_struct
            }
        }
    });
    println!("{}", i);
    i
}