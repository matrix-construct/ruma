//! Implementation of the `IdDst` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use crate::util::{RumaCommon, RumaCommonReexport};

mod parse;

/// Generate the `Owned` version of an identifier and various trait implementations.
pub(crate) fn expand_id_dst(input: syn::ItemStruct) -> syn::Result<TokenStream> {
    let id_dst = IdDst::parse(input)?;

    let id = &id_dst.types.id;

    let as_str_and_bytes_impls = id_dst.expand_as_str_and_bytes_impls();
    let to_string_impls = id_dst.expand_to_string_impls(id);
    let unchecked_from_str_impls = id_dst.expand_unchecked_from_str_impls();
    let owned_id_struct = id_dst.expand_owned_id_struct();
    let fallible_from_str_impls = id_dst.expand_fallible_from_str_impls();
    let infallible_from_str_impls = id_dst.expand_infallible_from_str_impls();
    let partial_eq_impls = id_dst.expand_partial_eq_impls();
    let zeroize_impl = id_dst.expand_zeroize_impl();

    Ok(quote! {
        #as_str_and_bytes_impls
        #to_string_impls
        #unchecked_from_str_impls
        #owned_id_struct
        #fallible_from_str_impls
        #infallible_from_str_impls
        #partial_eq_impls
        #zeroize_impl
    })
}

/// The parsed input of the `IdDst` macro.
struct IdDst {
    /// The name of the borrowed type.
    ident: syn::Ident,

    /// The name of the owned type.
    owned_ident: syn::Ident,

    /// The generics on the borrowed type.
    generics: syn::Generics,

    /// The declaration of the generics of the borrowed type to use on `impl` blocks.
    impl_generics: TokenStream,

    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// The index of the `str` field.
    ///
    /// This is assumed to be the last field of the tuple struct.
    str_field_index: syn::Index,

    /// Common types.
    types: Types,

    /// Inline-byte threshold for the `SmallVec` storage. Set per-type with
    /// `#[ruma_id(inline_bytes = N)]`; defaults to `DEFAULT_INLINE_BYTES`.
    inline_bytes: usize,

    /// The path to use imports from the ruma-common crate.
    ruma_common: RumaCommon,
}

impl IdDst {
    /// Generate `AsRef<str>` and `AsRef<[u8]>` implementations and string conversions for this
    /// identifier.
    fn expand_as_str_and_bytes_impls(&self) -> TokenStream {
        let ident = &self.ident;
        let impl_generics = &self.impl_generics;
        let str_field_index = &self.str_field_index;

        let str = &self.types.str;
        let bytes = &self.types.bytes;
        let string = &self.types.string;
        let id = &self.types.id;

        let as_str_docs = format!("Extracts a string slice from this `{ident}`.");
        let as_bytes_docs = format!("Extracts a byte slice from this `{ident}`.");

        quote! {
            impl #impl_generics #id {
                #[doc = #as_str_docs]
                #[inline]
                pub fn as_str(&self) -> &#str {
                    &self.#str_field_index
                }

                #[doc = #as_bytes_docs]
                #[inline]
                pub fn as_bytes(&self) -> &#bytes {
                    self.as_str().as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#str> for #id {
                fn borrow(&self) -> &#str {
                    self.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#id> for #id {
                fn as_ref(&self) -> &#id {
                    self
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#str> for #id {
                fn as_ref(&self) -> &#str {
                    self.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#bytes> for #id {
                fn as_ref(&self) -> &#bytes {
                    self.as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #string {
                fn from(id: &#id) -> Self {
                    id.as_str().to_owned()
                }
            }
        }
    }

    /// Generate unchecked private methods to convert a string type to the identifier.
    fn expand_unchecked_from_str_impls(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let id = &self.types.id;

        quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                pub(super) const fn from_borrowed_unchecked(s: &#str) -> &Self {
                    unsafe { ::std::mem::transmute(s) }
                }
            }
        }
    }

    /// Generate the `Owned{ident}` type and its implementations.
    fn expand_owned_id_struct(&self) -> TokenStream {
        let ident = &self.ident;
        let owned_ident = &self.owned_ident;
        let generics = &self.generics;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let bytes = &self.types.bytes;
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;
        let smallvec = self.types.smallvec_bytes(self.inline_bytes);

        let (phantom_decl, phantom_ctor) = if self.generics.params.is_empty() {
            None
        } else {
            let phantom_data = quote! { ::std::marker::PhantomData };
            let generic_types = generics.type_params().map(|param| &param.ident);

            Some((
                quote! { phantom: #phantom_data<( #(#generic_types,)* )>, },
                quote! { phantom: #phantom_data, },
            ))
        }
        .unzip();

        let doc_header = format!("Owned variant of [`{ident}`]");

        let to_string_impls = self.expand_to_string_impls(owned_id);

        let from_into_inner_impl = quote! {
            /// Consumes this identifier and returns its inner data.
            pub(super) fn into_inner(self) -> #smallvec {
                self.inner
            }

            /// Converts the inner data to this identifier, without checking that it is valid.
            ///
            /// # Safety
            ///
            /// This function is unsafe because it does not check that the data passed to it is
            /// valid for this identifier. If this constraint is violated, it may cause memory
            /// unsafety issues with future users of this type.
            pub(super) unsafe fn from_inner_unchecked(inner: #smallvec) -> Self {
                Self {
                    inner,
                    #phantom_ctor
                }
            }
        };

        quote! {
            #[doc = #doc_header]
            ///
            /// ## Inner representation
            ///
            /// Stores the identifier as a `SmallVec<[u8; INLINE_BYTES]>`: small identifiers
            /// (under `INLINE_BYTES`) live inline on the stack with no heap allocation, larger
            /// ones spill to the heap. The threshold is per-type, set with
            /// `#[ruma_id(inline_bytes = N)]`, defaulting to 32.
            pub struct #owned_ident #generics {
                inner: #smallvec,
                #phantom_decl
            }

            #[automatically_derived]
            impl #impl_generics #owned_id {
                pub(super) fn from_str_unchecked(s: &#str) -> Self {
                    Self {
                        inner: ::smallvec::SmallVec::from_slice(s.as_bytes()),
                        #phantom_ctor
                    }
                }

                pub(super) fn from_box_str_unchecked(s: #box_str) -> Self {
                    Self {
                        inner: ::smallvec::SmallVec::from_vec(::std::string::String::from(s).into_bytes()),
                        #phantom_ctor
                    }
                }

                pub(super) fn from_string_unchecked(s: #string) -> Self {
                    Self {
                        inner: ::smallvec::SmallVec::from_vec(s.into_bytes()),
                        #phantom_ctor
                    }
                }

                /// Access the inner string without going through the borrowed type.
                pub(super) fn as_inner_str(&self) -> &#str {
                    // SAFETY: validated as UTF-8 on construction; the SmallVec only ever holds
                    // bytes that came from a `&str` / `String` / `Box<str>`.
                    unsafe { ::std::str::from_utf8_unchecked(self.inner.as_slice()) }
                }

                /// Returns the byte length of this identifier.
                #[inline]
                pub fn len(&self) -> ::std::primitive::usize {
                    self.inner.len()
                }

                /// Returns `true` if this identifier has zero length.
                #[inline]
                pub fn is_empty(&self) -> ::std::primitive::bool {
                    self.inner.is_empty()
                }

                /// Returns the capacity of the underlying inline-or-heap buffer.
                #[inline]
                pub fn capacity(&self) -> ::std::primitive::usize {
                    self.inner.capacity()
                }

                /// Access the inner bytes without going through the borrowed type.
                pub(super) fn as_inner_bytes(&self) -> &#bytes {
                    self.inner.as_slice()
                }

                #from_into_inner_impl
            }

            #[automatically_derived]
            impl #impl_generics ::std::clone::Clone for #owned_id {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner_unchecked(self.inner.clone()) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #owned_id {
                fn eq(&self, other: &Self) -> bool {
                    self.inner.eq(&other.inner)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Eq for #owned_id {}

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialOrd for #owned_id {
                fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Ord for #owned_id {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.inner.cmp(&other.inner)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::hash::Hash for #owned_id {
                fn hash<H>(&self, state: &mut H)
                where
                    H: ::std::hash::Hasher,
                {
                    self.as_inner_str().hash(state)
                }
            }

            #to_string_impls

            #[automatically_derived]
            impl #impl_generics ::std::ops::Deref for #owned_id {
                type Target = #id;

                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#id> for #owned_id {
                fn borrow(&self) -> &#id {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#str> for #owned_id {
                fn borrow(&self) -> &#str {
                    self.as_inner_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#id> for #owned_id {
                fn as_ref(&self) -> &#id {
                    #ident::from_borrowed_unchecked(self.as_inner_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#str> for #owned_id {
                fn as_ref(&self) -> &#str {
                    self.as_inner_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#bytes> for #owned_id {
                fn as_ref(&self) -> &#bytes {
                    self.as_inner_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::ToOwned for #id {
                type Owned = #owned_id;

                fn to_owned(&self) -> Self::Owned {
                    #owned_ident::from_str_unchecked(self.as_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #owned_id {
                fn from(id: &#id) -> Self {
                    id.to_owned()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #box_str {
                fn from(id: #owned_id) -> Self {
                    // SAFETY: validated as UTF-8 on construction.
                    unsafe { ::std::string::String::from_utf8_unchecked(id.inner.into_vec()) }
                        .into_boxed_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #string {
                fn from(id: #owned_id) -> Self {
                    // SAFETY: validated as UTF-8 on construction.
                    unsafe { ::std::string::String::from_utf8_unchecked(id.inner.into_vec()) }
                }
            }
        }
    }

    /// Generate `FromStr` and other fallible string conversions implementations for this
    /// identifier, if it has a validation function.
    ///
    /// The error returned during conversion is `ruma_common::IdParseError`.
    fn expand_fallible_from_str_impls(&self) -> Option<TokenStream> {
        let validate = self.validate.as_ref()?;

        let ident = &self.ident;
        let owned_ident = &self.owned_ident;
        let generic_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);
        let serde_json = ruma_common.reexported(RumaCommonReexport::SerdeJson);

        let parse_doc_header = format!("Try parsing a `&str` into an `{owned_ident}`.");

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                #[doc = #parse_doc_header]
                ///
                /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
                /// This function is simply more constrained and thus useful in generic contexts.
                #[inline]
                pub fn parse(
                    s: impl ::std::convert::AsRef<#str>,
                ) -> ::std::result::Result<#owned_id, #ruma_common::IdParseError> {
                    let s = s.as_ref();
                    #validate(s)?;
                    ::std::result::Result::Ok(#owned_ident::from_str_unchecked(s))
                }

                /// Try parsing a `&str` into a borrowed reference of this identifier without
                /// allocating a new owned value.
                #[inline]
                pub fn parse_ref(s: &#str) -> ::std::result::Result<&Self, #ruma_common::IdParseError> {
                    #validate(s)?;
                    ::std::result::Result::Ok(#ident::from_borrowed_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics #owned_id {
                /// Try parsing a `&str` into an owned identifier.
                ///
                /// Convenience forwarder to the borrowed type's `parse`.
                #[inline]
                pub fn parse(
                    s: impl ::std::convert::AsRef<#str>,
                ) -> ::std::result::Result<Self, #ruma_common::IdParseError> {
                    #ident::parse(s)
                }

                /// Try assembling parts of an identifier (sigil, localpart, optional domain) into
                /// an owned identifier, without going through `format!`.
                pub fn from_parts(
                    sigil: ::std::primitive::char,
                    local: &#str,
                    domain: ::std::option::Option<&#str>,
                ) -> ::std::result::Result<Self, #ruma_common::IdParseError> {
                    let mut buf = [0u8; 4];
                    let sigil = sigil.encode_utf8(&mut buf);
                    let len = sigil.len() + local.len()
                        + domain.map(|d| d.len() + 1).unwrap_or(0);

                    let mut inner: ::smallvec::SmallVec<[::std::primitive::u8; #inline_bytes]> =
                        ::smallvec::SmallVec::with_capacity(len);
                    inner.extend_from_slice(sigil.as_bytes());
                    inner.extend_from_slice(local.as_bytes());
                    if let ::std::option::Option::Some(d) = domain {
                        inner.push(b':');
                        inner.extend_from_slice(d.as_bytes());
                    }

                    // SAFETY: all input came from `char` / `&str`, so the bytes are valid UTF-8.
                    let s = unsafe { ::std::str::from_utf8_unchecked(inner.as_slice()) };
                    #validate(s)?;

                    ::std::result::Result::Ok(unsafe { Self::from_inner_unchecked(inner) })
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<&'a #str> for &'a #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: &'a #str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(s)?;
                    ::std::result::Result::Ok(#ident::from_borrowed_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::str::FromStr for #owned_id {
                type Err = #ruma_common::IdParseError;

                fn from_str(s: &#str) -> ::std::result::Result<Self, Self::Err> {
                    #ident::parse(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<&#str> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: &#str) -> ::std::result::Result<Self, Self::Error> {
                    #ident::parse(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#box_str> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #box_str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(&s)?;
                    ::std::result::Result::Ok(#owned_ident::from_box_str_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#string> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #string) -> ::std::result::Result<Self, Self::Error> {
                    #validate(&s)?;
                    ::std::result::Result::Ok(#owned_ident::from_string_unchecked(s))
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<#cow_str> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #cow_str) -> ::std::result::Result<Self, Self::Error> {
                    match s {
                        #cow::Borrowed(s) => s.try_into(),
                        #cow::Owned(s) => s.try_into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #owned_id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    use #serde::de::Error;

                    // We always deserialize as a string to make sure that it is valid UTF-8,
                    // regardless of the inner representation.
                    #ruma_common::serde::deserialize_cow_str(deserializer)?
                        .try_into()
                        .map_err(D::Error::custom)
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for &'de #id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    use #serde::de::Error;

                    let s = <&'de #str>::deserialize(deserializer)?;
                    #validate(s).map_err(D::Error::custom)?;
                    ::std::result::Result::Ok(#ident::from_borrowed_unchecked(s))
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<&'a #serde_json::Value> for &'a #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(v: &'a #serde_json::Value) -> ::std::result::Result<Self, Self::Error> {
                    v.as_str().unwrap_or_default().try_into()
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<&'a #ruma_common::CanonicalJsonValue> for &'a #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(v: &'a #ruma_common::CanonicalJsonValue) -> ::std::result::Result<Self, Self::Error> {
                    v.as_str().unwrap_or_default().try_into()
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<::std::option::Option<&'a #serde_json::Value>> for &'a #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(v: ::std::option::Option<&'a #serde_json::Value>) -> ::std::result::Result<Self, Self::Error> {
                    v.and_then(|v| v.as_str()).unwrap_or_default().try_into()
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<::std::option::Option<&'a #ruma_common::CanonicalJsonValue>> for &'a #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(v: ::std::option::Option<&'a #ruma_common::CanonicalJsonValue>) -> ::std::result::Result<Self, Self::Error> {
                    v.and_then(|v| v.as_str()).unwrap_or_default().try_into()
                }
            }
        })
    }

    /// Generate `From<&str>` and other infallible string conversions implementations for this
    /// identifier, if it doesn't have a validation function.
    fn expand_infallible_from_str_impls(&self) -> Option<TokenStream> {
        if self.validate.is_some() {
            return None;
        }

        let ident = &self.ident;
        let owned_ident = &self.owned_ident;
        let impl_generics = &self.impl_generics;
        let generic_params = &self.generics.params;

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        Some(quote! {
            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<&'a #str> for &'a #id {
                fn from(s: &'a #str) -> Self {
                    #ident::from_borrowed_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#str> for #owned_id {
                fn from(s: &#str) -> Self {
                    #owned_ident::from_str_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_str> for #owned_id {
                fn from(s: #box_str) -> Self {
                    #owned_ident::from_box_str_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#string> for #owned_id {
                fn from(s: #string) -> Self {
                    #owned_ident::from_string_unchecked(s)
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<#cow_str> for #owned_id {
                fn from(s: #cow_str) -> Self {
                    match s {
                        #cow::Borrowed(s) => s.into(),
                        #cow::Owned(s) => s.into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #owned_id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    // We always deserialize as a string to make sure that it is valid UTF-8,
                    // regardless of the inner representation.
                    #ruma_common::serde::deserialize_cow_str(deserializer).map(::std::convert::Into::into)
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for &'de #id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    <&'de #str>::deserialize(deserializer).map(#ident::from_borrowed_unchecked)
                }
            }
        })
    }

    /// Generate `std::fmt::Display`, `std::fmt::Debug` or `serde::Serialize` traits
    /// implementations, using it's `.as_str()` function.
    fn expand_to_string_impls(&self, ty: &syn::Type) -> TokenStream {
        let serde = self.ruma_common.reexported(RumaCommonReexport::Serde);

        let impl_generics = &self.impl_generics;
        let str = &self.types.str;

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::fmt::Display for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::convert::AsRef::<#str>::as_ref(self).fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::fmt::Debug for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::convert::AsRef::<#str>::as_ref(self).fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics #serde::Serialize for #ty {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: #serde::Serializer,
                {
                    serializer.serialize_str(::std::convert::AsRef::<#str>::as_ref(self))
                }
            }
        }
    }

    /// Generate `std::cmp::PartialEq` implementations by comparing strings.
    fn expand_partial_eq_impls(&self) -> TokenStream {
        let generics_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

        let ref_id: syn::Type = parse_quote! { &#id };
        let ref_str: syn::Type = parse_quote! { &#str };
        let cow_generics = quote! { <'a, #generics_params> };

        // Implement `PartialEq` with the given lhs and rhs types.
        let expand_partial_eq = |lhs: &syn::Type, rhs: &syn::Type| {
            let impl_generics =
                if *lhs == *cow_str || *rhs == *cow_str { &cow_generics } else { impl_generics };

            quote! {
                #[automatically_derived]
                impl #impl_generics ::std::cmp::PartialEq<#rhs> for #lhs {
                    fn eq(&self, other: &#rhs) -> bool {
                        ::std::convert::AsRef::<#str>::as_ref(self) == ::std::convert::AsRef::<#str>::as_ref(other)
                    }
                }
            }
        };

        // Implement reciprocal `PartialEq` implementation for the given type with the given other
        // types.
        let expand_partial_eq_impls_for_type =
            |ty: &syn::Type, others: &[&syn::Type]| -> TokenStream {
                others
                    .iter()
                    .flat_map(|other| [expand_partial_eq(ty, other), expand_partial_eq(other, ty)])
                    .collect()
            };

        [
            expand_partial_eq_impls_for_type(id, &[str, &ref_str, string, cow_str]),
            expand_partial_eq_impls_for_type(
                owned_id,
                &[str, &ref_str, string, cow_str, id, &ref_id],
            ),
        ]
        .into_iter()
        .collect()
    }

    /// Generate the `zeroize` method for an owned type.
    fn expand_zeroize_impl(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;

        let owned_ident = &self.owned_ident;
        let owned_id = &self.types.owned_id;
        let parse_doc_header = format!(
            "Securely zero memory (aka [zeroize](https://en.wikipedia.org/wiki/Zeroisation)) of `{owned_ident}`."
        );

        quote! {
            #[automatically_derived]
            impl #impl_generics #owned_id {
                #[doc = #parse_doc_header]
                ///
                /// This method zeroizes this type by writing zeros in its
                /// memory location before freeing it. It internally uses
                /// [the `zeroize` crate][`zeroize`]. Note that this type
                /// doesn't implement the `zeroize::Zeroize` trait because the
                /// `Zeroize::zeroize` method takes a `&mut self`, which means
                /// we could put this type into an inconsistent state if it is
                /// used after calling that method. Instead, this method takes
                /// ownership of the type, ensuring it's impossible to misuse
                /// it.
                ///
                /// # Implementation details
                ///
                /// The owned identifier stores its bytes inline in a
                /// `SmallVec`, which are zeroized in place before the value is
                /// dropped.
                ///
                /// [`zeroize`]: https://docs.rs/zeroize/
                pub fn zeroize(mut self) {
                    ::zeroize::Zeroize::zeroize(self.inner.as_mut_slice());
                }
            }
        }
    }
}

/// Common types.
struct Types {
    /// `str`.
    str: syn::Type,

    /// `Cow`.
    cow: syn::Type,

    /// `Box<str>`.
    box_str: syn::Type,

    /// `String`.
    string: syn::Type,

    /// `Cow<'a, str>`.
    cow_str: syn::Type,

    /// `[u8]`.
    bytes: syn::Type,

    /// `{id}`, the identifier type with generics, if any.
    id: syn::Type,

    /// `{owned_id}`, the owned identifier type with generics, if any.
    owned_id: syn::Type,
}

impl Types {
    fn new(
        ident: &syn::Ident,
        owned_ident: &syn::Ident,
        type_generics: syn::TypeGenerics<'_>,
    ) -> Self {
        let str = parse_quote! { ::std::primitive::str };
        let cow = parse_quote! { ::std::borrow::Cow };

        let id = parse_quote! { #ident #type_generics };

        Self {
            box_str: parse_quote! { ::std::boxed::Box<#str> },
            string: parse_quote! { ::std::string::String },
            cow_str: parse_quote! { #cow<'a, #str> },
            bytes: parse_quote! { [::std::primitive::u8] },
            str,
            cow,
            id,
            owned_id: parse_quote! { #owned_ident #type_generics },
        }
    }

    /// `SmallVec<[u8; INLINE]>` for the per-type inline-byte threshold.
    fn smallvec_bytes(&self, inline_bytes: usize) -> syn::Type {
        parse_quote! { ::smallvec::SmallVec<[::std::primitive::u8; #inline_bytes]> }
    }
}

/// Default inline-byte threshold for the `SmallVec` storage representation.
pub(super) const DEFAULT_INLINE_BYTES: usize = 32;
