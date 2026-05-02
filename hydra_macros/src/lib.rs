use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{Arm, Error, Expr, ExprArray, ExprMatch, Ident, Item, ItemConst, ItemFn, Type, TypeArray, parse_macro_input, parse_quote, spanned::Spanned};

#[proc_macro_attribute]
pub fn bijective_array(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_ident = parse_macro_input!(attr as Ident);
    let fn_str = input_ident.to_string() + "_index";
    let fn_ident = Ident::new(&fn_str, Span::call_site());
    let item = parse_macro_input!(item as Item);

    let Item::Const(ItemConst{ ref expr, ref ty, .. }) = item else {
        return const_array_error(item.span());
    };
    let Type::Array(TypeArray { elem: ref elem_ty, .. }) = **ty else {
        return const_array_error(item.span());
    };

    let Expr::Array(ref arr) = **expr else {
        return const_array_error(expr.span());
    };

    let mut arms: Vec<Arm> = Vec::new();
    for (index, expr) in arr.elems.iter().enumerate() {
        arms.push(parse_quote!(
            #expr => #index
        ));
    }

    let panic_str = format!("index unavailable for {:?}", input_ident);
    let panic_str = panic_str.as_str();

    quote! {
        #item

        const fn #fn_ident(#input_ident: #elem_ty) -> usize {
            match #input_ident {
                #(#arms,)*
                _ => panic!("#")
            }
        }
    }.into()
}

fn const_array_error(span: Span) -> TokenStream {
    Error::new(span, "`bijective_array` attribute is only applicable to const array items").into_compile_error().into()
}