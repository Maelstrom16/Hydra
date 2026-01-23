use std::{fmt::Display, str::FromStr};

use funty::Integral;
use proc_macro::*;
use quote::{ToTokens, quote};
use syn::{Arm, Error, Expr, ExprStruct, Fields, FieldsNamed, Ident, ItemEnum, ItemStruct, LitInt, Stmt, Type, parse_macro_input, parse_quote, parse_str};

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

fn deserialized_register_helper<T>(item: TokenStream) -> TokenStream where T: Integral + ToTokens, <T as FromStr>::Err: Display {
    // Parse input struct
    let item_struct = parse_macro_input!(item as ItemStruct);
    let item_struct_ident = &item_struct.ident;

    // Throw an error if fields aren't named
    let Fields::Named(FieldsNamed { named: fields, .. }) = &item_struct.fields else {
        return Error::new_spanned(item_struct, "Only structs with named fields may be used as a deserialization target").to_compile_error().into();
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
        let width_result = field.attrs.iter()
            .find(|attr| attr.path().is_ident("width"))
            .ok_or(Error::new_spanned(field, "Missing `#[width(N)]` attribute"))
            .map(|attr| attr.parse_args::<LitInt>())
            .flatten()
            .map(|lit_int| lit_int.base10_parse::<T>())
            .flatten();
        let Ok(width) = width_result else {
            return width_result.unwrap_err().into_compile_error().into();
        };

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
            ty => {
                return Error::new_spanned(ty, "Unsupported type\nMust use `()`, `bool`, or an int type").into_compile_error().into();
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
        return Error::new_spanned(item_struct, format!("Total bit width must be {}", T::BITS)).into_compile_error().into()
    }

    // Final output
    let source_type = parse_str::<Type>(std::any::type_name::<T>()).unwrap();
    TokenStream::from(quote! {
        impl crate::common::bit::DeserializedRegister<#source_type> for #item_struct_ident {
            fn deserialize(value: #source_type) -> Self {
                #(#output_stmts)*
                #output_struct
            }
        }
    })
}

/// Attribute for enums where each variant represents a
/// trijective relationship between its identifier,
/// discriminant, and index 
#[proc_macro_derive(TrijectiveMMIO)] 
pub fn trijective_mmio(item: TokenStream) -> TokenStream {
    // Parse input enum
    let item_enum = parse_macro_input!(item as ItemEnum);
    let item_enum_ident = &item_enum.ident;
    let item_enum_string = item_enum_ident.to_string();

    // Retrieve repr type
    let repr_result = item_enum.attrs.iter()
        .find(|attr| attr.path().is_ident("repr"))
        .ok_or(Error::new_spanned(&item_enum, "missing `#[repr(T)]` attribute\n`#[repr(T)]` attribute is required to determine discriminants' types"))
        .map(|attr| attr.parse_args::<Type>())
        .flatten();
    let repr_type = match repr_result {
        Ok(ty) => ty,
        Err(e) => return e.into_compile_error().into()
    };

    // Ensure variants have discriminants, then attempt to build vectors of indices, variants, and discriminants
    let decomposition_result = item_enum.variants.iter()
        .enumerate()
        .map(|(index, variant)| variant.discriminant.as_ref()
            .ok_or(Error::new_spanned(&variant, "Missing discriminant"))
            .map(|(_, expr)| (index, &variant.ident, expr)))
        .collect::<Result<(Vec::<usize>, Vec::<&Ident>, Vec::<&Expr>), Error>>();
    let (indices, variants, discriminants) = match decomposition_result {
        Ok(o) => o,
        Err(e) => return e.into_compile_error().into()
    };

    // Initialize a few more necessary variables
    let variant_count = item_enum.variants.iter().count();
    let global_panic = format!("global address does not correspond to any {} variant", item_enum_string);
    let local_panic = format!("local address is out of bounds for {}", item_enum_string);

    // Final output
    let i = TokenStream::from(quote! {
        impl #item_enum_ident {
            pub const VARIANT_COUNT: usize = #variant_count;

            pub const fn to_global(&self) -> #repr_type {
                match self {
                    #(Self::#variants => #discriminants),*
                }
            }

            pub const fn to_local(&self) -> usize {
                match self {
                    #(Self::#variants => #indices),*
                }
            }

            pub const fn from_global(global: #repr_type) -> Self {
                match global {
                    #(#discriminants => Self::#variants,)*
                    invalid => panic!(#global_panic)
                }
            }

            pub const fn from_local(local: usize) -> Self {
                match local {
                    #(#indices => Self::#variants,)*
                    invalid => panic!(#local_panic)
                }
            }

            pub const fn global_to_local(global: #repr_type) -> usize {
                match global {
                    #(#discriminants => #indices,)*
                    invalid => panic!(#global_panic)
                }
            }

            pub const fn local_to_global(local: usize) -> #repr_type {
                match local {
                    #(#indices => #discriminants,)*
                    invalid => panic!(#local_panic)
                }
            }
        }
    });
    println!("{}", i);
    i
}