//! Proc macros for the Polymarket client SDK.
//!
//! This crate provides procedural macros to reduce boilerplate when working with
//! enums that need to handle unknown API values gracefully.
//!
//! # `#[unknown_enum_variant]`
//!
//! Automatically adds an `Unknown(String)` variant to an enum and implements
//! both the `UnknownEnumVariant` trait and a custom `Deserialize` that warns
//! when an unknown variant is encountered.
//!
//! ## Example
//!
//! ```ignore
//! use polymarket_client_sdk::unknown_enum_variant;
//!
//! #[unknown_enum_variant]
//! #[derive(Debug, Serialize)]  // Note: Deserialize is auto-generated
//! #[serde(rename_all = "UPPERCASE")]
//! pub enum OrderType {
//!     GTC,
//!     FOK,
//! }
//! ```
//!
//! The macro:
//! - Adds an `Unknown(String)` variant
//! - Implements `UnknownEnumVariant` trait
//! - Generates a `Deserialize` impl that logs a warning when unknown variants are encountered

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Fields, Ident, Lit, Meta, Variant, parse::Parse,
    parse_macro_input, parse_quote,
};

/// Automatically adds an `Unknown(String)` variant and implements `UnknownEnumVariant`.
///
/// This attribute macro transforms an enum to support forward-compatible deserialization
/// by adding an `Unknown(String)` variant that catches any unrecognized values from the API.
///
/// # Behavior
///
/// - Adds an `Unknown(String)` variant (or validates existing one)
/// - Implements the `UnknownEnumVariant` trait
/// - Generates a custom `Deserialize` impl that warns on unknown variants
/// - Removes `Deserialize` from derives if present (since we generate it)
///
/// # Requirements
///
/// - Must be applied to an enum (not a struct or union)
/// - If an `Unknown` variant already exists, it must be `Unknown(String)`
///
/// # Example
///
/// ```ignore
/// #[unknown_enum_variant]
/// #[derive(Debug, Clone, Serialize)]
/// #[serde(rename_all = "UPPERCASE")]
/// pub enum OrderType {
///     GTC,
///     FOK,
///     GTD,
/// }
/// ```
#[proc_macro_attribute]
pub fn unknown_enum_variant(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match impl_unknown_enum_variant(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// The rename_all casing strategy from serde.
#[derive(Debug, Clone, Copy, Default)]
enum RenameAll {
    #[default]
    None,
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameAll {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(Self::Lowercase),
            "UPPERCASE" => Some(Self::Uppercase),
            "PascalCase" => Some(Self::PascalCase),
            "camelCase" => Some(Self::CamelCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(Self::ScreamingKebabCase),
            _ => None,
        }
    }

    fn apply(&self, name: &str) -> String {
        match self {
            Self::None | Self::PascalCase => name.to_owned(),
            Self::Lowercase => name.to_lowercase(),
            Self::Uppercase => name.to_uppercase(),
            Self::CamelCase => {
                let mut c = name.chars();
                match c.next() {
                    Some(first) => first.to_lowercase().chain(c).collect(),
                    None => String::new(),
                }
            }
            Self::SnakeCase => to_snake_case(name),
            Self::ScreamingSnakeCase => to_snake_case(name).to_uppercase(),
            Self::KebabCase => to_snake_case(name).replace('_', "-"),
            Self::ScreamingKebabCase => to_snake_case(name).to_uppercase().replace('_', "-"),
        }
    }
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

fn impl_unknown_enum_variant(mut input: DeriveInput) -> Result<TokenStream2, Error> {
    // Validate that this is an enum
    let data_enum = match &mut input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "#[unknown_enum_variant] can only be applied to enums",
            ));
        }
    };

    let enum_name = &input.ident;

    // Parse serde attributes to get rename_all
    let rename_all = parse_rename_all(&input.attrs);

    // Collect variant info before modifying
    let variants_info: Vec<VariantInfo> = data_enum
        .variants
        .iter()
        .filter(|v| v.ident != "Unknown")
        .map(|v| VariantInfo::from_variant(v, rename_all))
        .collect();

    // Check if Unknown variant already exists
    let has_unknown = data_enum.variants.iter().any(|v| v.ident == "Unknown");

    if has_unknown {
        // Validate existing Unknown variant has correct structure
        validate_existing_unknown_variant(&data_enum.variants, enum_name)?;
    } else {
        // Add Unknown(String) variant
        add_unknown_variant(data_enum);
    }

    // Remove Deserialize from derives (we generate our own)
    remove_deserialize_derive(&mut input.attrs);

    // Generate the trait implementation
    let trait_impl = generate_trait_impl(enum_name);

    // Generate the Deserialize implementation
    let deserialize_impl = generate_deserialize_impl(enum_name, &variants_info);

    // Combine enum definition and impls
    let expanded = quote! {
        #input
        #trait_impl
        #deserialize_impl
    };

    Ok(expanded)
}

struct VariantInfo {
    ident: Ident,
    serialized_name: String,
    aliases: Vec<String>,
}

impl VariantInfo {
    fn from_variant(variant: &Variant, rename_all: RenameAll) -> Self {
        let ident = variant.ident.clone();
        let mut serialized_name = rename_all.apply(&ident.to_string());
        let mut aliases = Vec::new();

        // Check for #[serde(rename = "...")] and #[serde(alias = "...")]
        for attr in &variant.attrs {
            if attr.path().is_ident("serde")
                && let Meta::List(meta_list) = &attr.meta
                && let Ok(nested) = meta_list.parse_args_with(|input: syn::parse::ParseStream| {
                    input.parse_terminated(Meta::parse, syn::token::Comma)
                })
            {
                for meta in nested {
                    if let Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("rename")
                            && let Expr::Lit(expr_lit) = &nv.value
                            && let Lit::Str(lit_str) = &expr_lit.lit
                        {
                            serialized_name = lit_str.value();
                        } else if nv.path.is_ident("alias")
                            && let Expr::Lit(expr_lit) = &nv.value
                            && let Lit::Str(lit_str) = &expr_lit.lit
                        {
                            aliases.push(lit_str.value());
                        }
                    }
                }
            }
        }

        VariantInfo {
            ident,
            serialized_name,
            aliases,
        }
    }
}

fn parse_rename_all(attrs: &[Attribute]) -> RenameAll {
    for attr in attrs {
        if attr.path().is_ident("serde")
            && let Meta::List(meta_list) = &attr.meta
            && let Ok(nested) = meta_list.parse_args_with(|input: syn::parse::ParseStream| {
                input.parse_terminated(Meta::parse, syn::token::Comma)
            })
        {
            for meta in nested {
                if let Meta::NameValue(nv) = meta
                    && nv.path.is_ident("rename_all")
                    && let Expr::Lit(expr_lit) = &nv.value
                    && let Lit::Str(lit_str) = &expr_lit.lit
                    && let Some(ra) = RenameAll::from_str(&lit_str.value())
                {
                    return ra;
                }
            }
        }
    }
    RenameAll::None
}

fn remove_deserialize_derive(attrs: &mut [Attribute]) {
    for attr in attrs.iter_mut() {
        if attr.path().is_ident("derive")
            && let Meta::List(meta_list) = &attr.meta
            && let Ok(paths) = meta_list.parse_args_with(|input: syn::parse::ParseStream| {
                input.parse_terminated(syn::Path::parse, syn::token::Comma)
            })
        {
            let filtered: Vec<_> = paths
                .into_iter()
                .filter(|path| {
                    path.segments
                        .last()
                        .is_none_or(|seg| seg.ident != "Deserialize")
                })
                .collect();

            // Rebuild the attribute with proper comma separation
            *attr = parse_quote!(#[derive(#(#filtered),*)]);
        }
    }
}

/// Add the `Unknown(String)` variant.
fn add_unknown_variant(data_enum: &mut syn::DataEnum) {
    let variant: Variant = parse_quote! {
        /// Unknown variant from the API (captures the raw value for debugging).
        Unknown(String)
    };

    data_enum.variants.push(variant);
}

/// Validate that an existing `Unknown` variant has the correct structure.
fn validate_existing_unknown_variant(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    enum_name: &Ident,
) -> Result<(), Error> {
    let unknown_variant = variants
        .iter()
        .find(|v| v.ident == "Unknown")
        .expect("Unknown variant should exist");

    // Check that it's a tuple variant with one String field
    match &unknown_variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let field = &fields.unnamed[0];
            // Validate field type is String (best-effort - doesn't handle type aliases)
            if let syn::Type::Path(type_path) = &field.ty
                && type_path
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "String")
            {
                return Ok(());
            }
            Err(Error::new_spanned(
                unknown_variant,
                format!(
                    "{enum_name} already has an Unknown variant, but it must be Unknown(String)"
                ),
            ))
        }
        _ => Err(Error::new_spanned(
            unknown_variant,
            format!("{enum_name} already has an Unknown variant, but it must be Unknown(String)"),
        )),
    }
}

/// Generate the `UnknownEnumVariant` trait implementation.
fn generate_trait_impl(enum_name: &Ident) -> TokenStream2 {
    let type_name_str = enum_name.to_string();

    quote! {
        impl crate::serde_helpers::UnknownEnumVariant for #enum_name {
            fn as_unknown(&self) -> Option<&str> {
                match self {
                    #enum_name::Unknown(s) => Some(s),
                    _ => None,
                }
            }

            fn type_name() -> &'static str {
                #type_name_str
            }
        }
    }
}

/// Generate the `Deserialize` implementation with warning on unknown variants.
fn generate_deserialize_impl(enum_name: &Ident, variants: &[VariantInfo]) -> TokenStream2 {
    let type_name_str = enum_name.to_string();

    // Generate match arms for known variants
    let match_arms: Vec<TokenStream2> = variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let name = &v.serialized_name;
            let aliases = &v.aliases;

            if aliases.is_empty() {
                quote! {
                    #name => ::core::result::Result::Ok(#enum_name::#ident),
                }
            } else {
                quote! {
                    #name #(| #aliases)* => ::core::result::Result::Ok(#enum_name::#ident),
                }
            }
        })
        .collect();

    // Create the expected variants string for error message
    let expected: Vec<&str> = variants
        .iter()
        .map(|v| v.serialized_name.as_str())
        .collect();
    let expected_str = expected.join(", ");

    // Create a unique visitor name to avoid conflicts
    let visitor_name = Ident::new(&format!("{enum_name}Visitor"), Span::call_site());

    quote! {
        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct #visitor_name;

                impl serde::de::Visitor<'_> for #visitor_name {
                    type Value = #enum_name;

                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        formatter.write_str(concat!("a string for ", #type_name_str))
                    }

                    fn visit_str<E>(self, value: &str) -> ::core::result::Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            #(#match_arms)*
                            unknown => {
                                #[cfg(feature = "tracing")]
                                tracing::warn!(
                                    enum_type = #type_name_str,
                                    unknown_value = %unknown,
                                    expected = #expected_str,
                                    "unknown enum variant in API response"
                                );
                                ::core::result::Result::Ok(#enum_name::Unknown(unknown.to_owned()))
                            }
                        }
                    }
                }

                deserializer.deserialize_str(#visitor_name)
            }
        }
    }
}
