use std::{fmt::Display, str::FromStr};

use funty::Integral;
use proc_macro::*;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{Error, Expr, Fields, Ident, ItemEnum, ItemImpl, ItemStruct, LitInt, Type, parse_macro_input, parse_quote, parse_str};

/// Attribute for structs which can be deserialized from an int register.
#[proc_macro_attribute]
pub fn field_map(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse input arguments and struct
    let ty = parse_macro_input!(attr as Ident);
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    let item_impl_result = match ty.to_string().as_str() {
        "u8" => field_map_impl_helper::<u8>(&item_struct),
        "u16" => field_map_impl_helper::<u16>(&item_struct),
        "u32" => field_map_impl_helper::<u32>(&item_struct),
        "u64" => field_map_impl_helper::<u64>(&item_struct),
        _ => return Error::new_spanned(ty, "invalid type\nmust use one of `u8`, `u16`, `u32`, or `u64`").into_compile_error().into()
    };
    let item_impl = match item_impl_result {
        Ok(item_impl) => item_impl,
        Err(e) => return e.into_compile_error().into()
    };

    item_struct.fields = Fields::Named(parse_quote!({inner: std::rc::Rc<crate::common::bit::MaskedBitSet<#ty>>}));
    let item_struct_ident = &item_struct.ident;
    
    TokenStream::from(quote! {
        #item_struct
        // Constructor
        impl #item_struct_ident {
            /// Wraps a `MaskedBitSet<T>` for use with this field map.
            pub fn wrap(bitset: std::rc::Rc<crate::common::bit::MaskedBitSet<#ty>>) -> #item_struct_ident {
                #item_struct_ident { inner: bitset, }
            }
        }
        // Accessors
        #item_impl
    })
}

fn field_map_impl_helper<RegT>(item_struct: &ItemStruct) -> Result<ItemImpl, Error> where RegT: Integral + ToTokens, <RegT as FromStr>::Err: Display{
    let mut final_output = TokenStream2::new();

    let mut cumulative_width: RegT = RegT::ZERO;
    for field in item_struct.fields.iter().rev() {
        let width = field.attrs.iter()
            .find(|attr| attr.path().is_ident("width"))
            .ok_or(Error::new_spanned(&field, "no `#[width(N)]` attribute found"))
            .and_then(|attr| attr.parse_args::<LitInt>())
            .and_then(|lit_int| lit_int.base10_parse::<RegT>())?;
        cumulative_width += width;

        let reg_ty = parse_str::<Type>(std::any::type_name::<RegT>()).unwrap();
        let field_ty = &field.ty;
        let (get_expr, set_expr): (Expr, Expr) = match field_ty {
            Type::Path(path) => {
                let bitmask = RegT::ONE.checked_shl(width.as_u32()).unwrap_or(RegT::ZERO).wrapping_sub(RegT::ONE);
                let shift_width = cumulative_width - width;
                let (bool_ineq, bool_cast) = if path.path.is_ident("bool") {
                    (quote! {!= 0}, quote! {as #reg_ty})
                } else {
                    (TokenStream2::new(), TokenStream2::new())
                };

                (
                    parse_quote! {(self.inner.get() >> #shift_width) & #bitmask #bool_ineq},
                    parse_quote! {self.inner.set((self.inner.get() & !(#bitmask << #shift_width)) | ((val #bool_cast & #bitmask) << #shift_width))}
                )
            }
            Type::Never(_) => continue, // No further processing required for padding
            _ => return Err(Error::new_spanned(field_ty, "unsupported type"))
        };
        
        let ident_suffix = (item_struct.fields.len() != 1).then_some(
            field.ident.as_ref()
            .ok_or(Error::new_spanned(&item_struct, "`field_map` attribute invalid for tuple structs\nnamed fields are required to build access functions"))?
            .to_string()
        );
        let (get_ident, set_ident) = accessor_idents(ident_suffix);
        
        final_output.extend(quote! {
            pub fn #get_ident(&self) -> #field_ty {
                #get_expr
            }
            pub fn #set_ident(&mut self, val: #field_ty) {
                #set_expr
            }
        });
    }

    let item_struct_ident = &item_struct.ident;
    Ok(parse_quote! {
        impl #item_struct_ident {
            #final_output
        }
    })
}

fn accessor_idents(suffix: Option<String>) -> (Ident, Ident) {
    let ident_string = suffix.map_or_else(String::new, |str| "_".to_owned() + &str);
    let ident_fn = |prefix: &str| Ident::new((prefix.to_owned() + &ident_string).as_str(), Span::call_site());

    (ident_fn("get"), ident_fn("set"))
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
        .and_then(|attr| attr.parse_args::<Type>());
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
    TokenStream::from(quote! {
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
    })
}