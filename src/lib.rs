use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Data, DataEnum, DataStruct, DataUnion,
    DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Path, Token,
};

/// Takes the struct, enum, or union it is applied to, and calls every macro provided on it.
///
/// # Usage
///
/// ```rust
/// # macro_rules! macro0 { ($($any:tt)*) => {} }
/// # macro_rules! macro1 { ($($any:tt)*) => {} }
/// # use derive2::derive2;
///
/// #[derive2(macro0, macro1)]
/// #[derive(Default)]
/// pub enum Foo {
///     #[default]
///     Bar,
///     #[macro0(something)]
///     Baz,
/// }
/// ```
///
/// The above expands to:
/// ```rust
/// # macro_rules! macro0 { ($($any:tt)*) => {} }
/// # macro_rules! macro1 { ($($any:tt)*) => {} }
/// # use derive2::derive2;
///
/// #[derive(Default)]
/// pub enum Foo {
///     #[default]
///     Bar,
///     Baz,
/// }
///
/// macro0! {
///     #[derive(Default)]
///     pub enum Foo {
///         #[default]
///         Bar,
///         #[macro0(something)]
///         Baz,
///     }
/// }
/// macro1! {
///     #[derive(Default)]
///     pub enum Foo {
///         #[default]
///         Bar,
///         #[macro0(something)]
///         Baz,
///     }
/// }
/// ```
///
/// Notice that the macro0 attribute attached to Foo::Baz is missing from the Foo declaration. Any
/// attribute which starts with macro0 or macro1 is removed from the resulting declaration. This
/// would otherwise error out.
#[proc_macro_attribute]
pub fn derive2(args: TokenStream, input: TokenStream) -> TokenStream {
    let reemit_tokens: TokenStream2 = input.clone().into();
    let mut input = syn::parse_macro_input!(input as DeriveInput);
    let args_parsed = syn::parse_macro_input!(args as Derive2Args);

    let macro_names = args_parsed.args.into_iter().collect::<Vec<_>>();

    retain_unrecognised_attrs(&mut input.attrs, &macro_names);

    match &mut input.data {
        // Matches all the field variants (named struct, unnamed struct, union)
        Data::Struct(DataStruct {
            fields:
                Fields::Named(FieldsNamed { named: fields, .. })
                | Fields::Unnamed(FieldsUnnamed {
                    unnamed: fields, ..
                }),
            ..
        })
        | Data::Union(DataUnion {
            fields: FieldsNamed { named: fields, .. },
            ..
        }) => {
            fields
                .iter_mut()
                .for_each(|f| retain_unrecognised_attrs(&mut f.attrs, &macro_names));
        }
        Data::Enum(DataEnum { variants, .. }) => {
            variants
                .iter_mut()
                .for_each(|v| retain_unrecognised_attrs(&mut v.attrs, &macro_names));
        }
        // Unit structs
        _ => (),
    }

    quote::quote! {
        #input

        #(#macro_names! {
            #reemit_tokens
        })*
    }
    .into()
}

struct Derive2Args {
    args: Punctuated<Path, Token![,]>,
}

impl Parse for Derive2Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: input.parse_terminated(Path::parse, Token![,])?,
        })
    }
}

/// Only retains attributes which do not match the final path segment of any of the paths in matches
fn retain_unrecognised_attrs(attrs: &mut Vec<Attribute>, matches: &[Path]) {
    attrs.retain(|attr| {
        if let Some(ident) = attr.meta.path().segments.first().map(|s| &s.ident) {
            // Only remove the attr if the meta matches one of the macros names
            matches.iter().all(|mn| match mn.segments.last() {
                Some(segment) if &segment.ident == ident => false,
                _ => true,
            })
        } else {
            true
        }
    });
}
