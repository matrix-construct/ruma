//! Methods and types for generating identifiers.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Fields, ImplGenerics, Index, ItemStruct, LitStr, Path, Token,
};

pub struct IdentifierInput {
    pub dollar_crate: Path,
    pub id: LitStr,
}

impl Parse for IdentifierInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let dollar_crate = input.parse()?;
        let _: Token![,] = input.parse()?;
        let id = input.parse()?;

        Ok(Self { dollar_crate, id })
    }
}

pub fn expand_id_dst(input: ItemStruct) -> syn::Result<TokenStream> {
    let meta = input.attrs.iter().filter(|attr| attr.path().is_ident("ruma_id")).try_fold(
        IdDstMeta::default(),
        |meta, attr| {
            let list: Punctuated<IdDstMeta, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;

            list.into_iter().try_fold(meta, IdDstMeta::merge)
        },
    )?;

    let extra_impls = if let Some(validate) = meta.validate {
        expand_checked_impls(&input, validate)
    } else {
        assert!(
            input.generics.params.is_empty(),
            "generic unchecked IDs are not currently supported"
        );
        expand_unchecked_impls(&input)
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // So we don't have to insert #where_clause everywhere when it is always None in practice
    assert_eq!(where_clause, None, "where clauses on identifier types are not currently supported");

    let id = &input.ident;
    let owned = format_ident!("Owned{id}");
    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    const INLINE_BYTES: usize = 32;
    let inline_bytes = meta.inline_bytes.unwrap_or(INLINE_BYTES);
    let inline_array = quote! { [u8; #inline_bytes] };
    let sv_decl = quote! { smallvec::SmallVec<#inline_array> };
    let sv = quote! { smallvec::SmallVec::<#inline_array> };

    let owned_decl = expand_owned_id(&input, inline_bytes);

    let as_str_impl = match &input.fields {
        Fields::Named(_) | Fields::Unit => {
            syn::Error::new(Span::call_site(), "Only tuple structs are supported currently.")
                .into_compile_error()
        }
        Fields::Unnamed(u) => {
            let last_idx = Index::from(u.unnamed.len() - 1);
            quote! { &self.#last_idx }
        }
    };

    let as_str_impls = expand_as_str_impls(id_ty.clone(), &impl_generics);
    // FIXME: Remove?
    let box_partial_eq_string = expand_partial_eq_string(quote! { Box<#id_ty> }, &impl_generics);

    let as_str_docs = format!("Creates a string slice from this `{id}`.");
    let as_bytes_docs = format!("Creates a byte slice from this `{id}`.");
    let _max_bytes_docs = format!("Maximum byte length for any `{id}`.");

    Ok(quote! {
        #owned_decl

        #[automatically_derived]
        impl #impl_generics #id_ty {
            pub(super) const fn from_borrowed(s: &str) -> &Self {
                unsafe { std::mem::transmute(s) }
            }

            pub(super) fn from_box(s: Box<str>) -> Box<Self> {
                unsafe { Box::from_raw(Box::into_raw(s) as _) }
            }

            pub(super) fn from_rc(s: std::rc::Rc<str>) -> std::rc::Rc<Self> {
                unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(s) as _) }
            }

            pub(super) fn from_arc(s: std::sync::Arc<str>) -> std::sync::Arc<Self> {
                unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(s) as _) }
            }

            pub(super) fn into_owned(self: Box<Self>) -> #sv_decl {
                let len = self.as_bytes().len();
                let p: *mut u8 = Box::into_raw(self).cast();
                let v = unsafe { Vec::<u8>::from_raw_parts(p, len, len) };
                #sv::from_vec(v)
            }

            pub(super) fn into_box(s: Box<Self>) -> Box<str> {
                unsafe { Box::from_raw(Box::into_raw(s) as _) }
            }

            #[doc = #as_str_docs]
            #[inline]
            pub fn as_str(&self) -> &str {
                #as_str_impl
            }

            #[doc = #as_bytes_docs]
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                self.as_str().as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics Clone for Box<#id_ty> {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        #[automatically_derived]
        impl #impl_generics ToOwned for #id_ty {
            type Owned = #owned_ty;

            fn to_owned(&self) -> Self::Owned {
                Self::Owned::new(self.as_bytes().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<#id_ty> for #id_ty {
            fn as_ref(&self) -> &#id_ty {
                self
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for #id_ty {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for Box<#id_ty> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<[u8]> for #id_ty {
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<[u8]> for Box<#id_ty> {
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for String {
            fn from(id: &#id_ty) -> Self {
                id.as_str().to_owned()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<#id_ty>> for String {
            fn from(id: Box<#id_ty>) -> Self {
                String::from(<#id_ty>::into_box(id))
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for Box<#id_ty> {
            fn from(id: &#id_ty) -> Self {
                <#id_ty>::from_box(id.as_str().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for std::rc::Rc<#id_ty> {
            fn from(s: &#id_ty) -> std::rc::Rc<#id_ty> {
                let rc = std::rc::Rc::<str>::from(s.as_str());
                <#id_ty>::from_rc(rc)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for std::sync::Arc<#id_ty> {
            fn from(s: &#id_ty) -> std::sync::Arc<#id_ty> {
                let arc = std::sync::Arc::<str>::from(s.as_str());
                <#id_ty>::from_arc(arc)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#id_ty> for Box<#id_ty> {
            fn eq(&self, other: &#id_ty) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<&'_ #id_ty> for Box<#id_ty> {
            fn eq(&self, other: &&#id_ty) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<Box<#id_ty>> for #id_ty {
            fn eq(&self, other: &Box<#id_ty>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<Box<#id_ty>> for &'_ #id_ty {
            fn eq(&self, other: &Box<#id_ty>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #as_str_impls
        #box_partial_eq_string
        #extra_impls
    })
}

fn expand_owned_id(input: &ItemStruct, inline_bytes: usize) -> TokenStream {
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let listed_generics: Punctuated<_, Token![,]> =
        input.generics.type_params().map(|param| &param.ident).collect();

    let id = &input.ident;
    let owned = format_ident!("Owned{id}");
    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    let inline_array = quote! { [u8; #inline_bytes] };
    let sv_decl = quote! { smallvec::SmallVec<#inline_array> };

    let has_generics = !listed_generics.is_empty();
    let phantom_decl = if has_generics {
        quote! { _p: std::marker::PhantomData<(#listed_generics)>, }
    } else {
        quote! {}
    };

    let phantom_impl = if has_generics {
        quote! { _p: std::marker::PhantomData::<(#listed_generics)>, }
    } else {
        quote! {}
    };

    let as_str_impls = expand_as_str_impls(owned_ty.clone(), &impl_generics);

    let doc_header = format!("Owned variant of {id}");

    quote! {
        #[doc = #doc_header]
        pub struct #owned #impl_generics {
            inner: #sv_decl,
            #phantom_decl
        }

        #[automatically_derived]
        impl #impl_generics #owned_ty {
            fn new(inner: #sv_decl) -> Self {
                Self {
                    inner,
                    #phantom_impl
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<#id_ty> for #owned_ty {
            fn as_ref(&self) -> &#id_ty {
                let s: &str = self.as_ref();
                <#id_ty>::from_borrowed(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for #owned_ty {
            fn as_ref(&self) -> &str {
                let s: &[u8] = self.as_ref();
                unsafe { std::str::from_utf8_unchecked(s) }
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<[u8]> for #owned_ty {
            fn as_ref(&self) -> &[u8] {
                self.inner.as_slice()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#owned_ty> for String {
            fn from(id: #owned_ty) -> String {
                unsafe { String::from_utf8_unchecked(id.inner.into_vec()) }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::clone::Clone for #owned_ty {
            fn clone(&self) -> Self {
                Self::new(self.inner.clone())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::ops::Deref for #owned_ty {
            type Target = #id_ty;

            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        #[automatically_derived]
        impl #impl_generics std::borrow::Borrow<#id_ty> for #owned_ty {
            fn borrow(&self) -> &#id_ty {
                self.as_ref()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&'_ #id_ty> for #owned_ty {
            fn from(id: &#id_ty) -> #owned_ty {
                Self::new(id.as_bytes().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<#id_ty>> for #owned_ty {
            fn from(b: Box<#id_ty>) -> #owned_ty {
                Self::new(<#id_ty>::into_owned(b))
            }
        }

        #[automatically_derived]
        impl #impl_generics From<std::sync::Arc<#id_ty>> for #owned_ty {
            fn from(a: std::sync::Arc<#id_ty>) -> #owned_ty {
                Self::new(a.as_bytes().into()) //TODO
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#owned_ty> for Box<#id_ty> {
            fn from(a: #owned_ty) -> Box<#id_ty> {
                { Box::from(<#id_ty>::from_borrowed(a.as_str())) }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#owned_ty> for std::sync::Arc<#id_ty> {
            fn from(a: #owned_ty) -> std::sync::Arc<#id_ty> {
                { std::sync::Arc::from(<#id_ty>::from_borrowed(a.as_str())) }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::cmp::PartialEq for #owned_ty {
            fn eq(&self, other: &Self) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics std::cmp::Eq for #owned_ty {}

        #[automatically_derived]
        impl #impl_generics std::cmp::PartialOrd for #owned_ty {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        #[automatically_derived]
        impl #impl_generics std::cmp::Ord for #owned_ty {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::hash::Hash for #owned_ty {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.as_str().hash(state)
            }
        }

        #as_str_impls

        #[automatically_derived]
        impl #impl_generics PartialEq<#id_ty> for #owned_ty {
            fn eq(&self, other: &#id_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == other
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for #id_ty {
            fn eq(&self, other: &#owned_ty) -> bool {
                self == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<&#id_ty> for #owned_ty {
            fn eq(&self, other: &&#id_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == *other
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for &#id_ty {
            fn eq(&self, other: &#owned_ty) -> bool {
                *self == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<Box<#id_ty>> for #owned_ty {
            fn eq(&self, other: &Box<#id_ty>) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for Box<#id_ty> {
            fn eq(&self, other: &#owned_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<std::sync::Arc<#id_ty>> for #owned_ty {
            fn eq(&self, other: &std::sync::Arc<#id_ty>) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for std::sync::Arc<#id_ty> {
            fn eq(&self, other: &#owned_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }
    }
}

fn expand_checked_impls(input: &ItemStruct, validate: Path) -> TokenStream {
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let generic_params = &input.generics.params;

    let id = &input.ident;
    let owned = format_ident!("Owned{id}");
    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    let parse_doc_header = format!("Try parsing a `&str` into an `Owned{id}`.");
    let parse_box_doc_header = format!("Try parsing a `&str` into a `Box<{id}>`.");
    let parse_rc_docs = format!("Try parsing a `&str` into an `Rc<{id}>`.");
    let parse_arc_docs = format!("Try parsing a `&str` into an `Arc<{id}>`.");

    quote! {
        #[automatically_derived]
        impl #impl_generics #id_ty {
            #[inline]
            #[doc = #parse_doc_header]
            ///
            /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
            /// This function is simply more constrained and thus useful in generic contexts.
            pub fn parse(
                s: impl AsRef<str>,
            ) -> Result<#owned_ty, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok((<#id_ty>::from_borrowed(s.as_ref())).to_owned())
            }

            #[inline]
            #[doc = #parse_box_doc_header]
            ///
            /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
            /// This function is simply more constrained and thus useful in generic contexts.
            pub fn parse_box(
                s: impl AsRef<str> + Into<Box<str>>,
            ) -> Result<Box<Self>, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok(<#id_ty>::from_box(s.into()))
            }

            #[inline]
            #[doc = #parse_rc_docs]
            pub fn parse_rc(
                s: impl AsRef<str> + Into<std::rc::Rc<str>>,
            ) -> Result<std::rc::Rc<Self>, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok(<#id_ty>::from_rc(s.into()))
            }

            #[inline]
            #[doc = #parse_arc_docs]
            pub fn parse_arc(
                s: impl AsRef<str> + Into<std::sync::Arc<str>>,
            ) -> Result<std::sync::Arc<Self>, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok(<#id_ty>::from_arc(s.into()))
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for Box<#id_ty> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match <#id_ty>::parse_box(s) {
                    Ok(o) => Ok(o),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for #owned_ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match <#id_ty>::parse(s) {
                    Ok(o) => Ok(o),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for &'de #id_ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = <&'de str>::deserialize(deserializer)?;

                match #validate(s) {
                    Ok(_) => Ok(<#id_ty>::from_borrowed(s)),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        #[automatically_derived]
        impl<'a, #generic_params> std::convert::TryFrom<&'a str> for &'a #id_ty {
            type Error = crate::IdParseError;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                #validate(s)?;
                Ok(<#id_ty>::from_borrowed(s))
            }
        }

        #[automatically_derived]
        impl<'a, #generic_params> std::convert::TryFrom<&'a serde_json::Value> for &'a #id_ty {
            type Error = crate::IdParseError;

            fn try_from(v: &'a serde_json::Value) -> Result<Self, Self::Error> {
                v.as_str().unwrap_or_default().try_into()
            }
        }

        #[automatically_derived]
        impl<'a, #generic_params> std::convert::TryFrom<&'a crate::CanonicalJsonValue> for &'a #id_ty {
            type Error = crate::IdParseError;

            fn try_from(v: &'a crate::CanonicalJsonValue) -> Result<Self, Self::Error> {
                v.as_str().unwrap_or_default().try_into()
            }
        }

        #[automatically_derived]
        impl<'a, #generic_params> std::convert::TryFrom<Option<&'a serde_json::Value>> for &'a #id_ty {
            type Error = crate::IdParseError;

            fn try_from(v: Option<&'a serde_json::Value>) -> Result<Self, Self::Error> {
                v.and_then(|v| v.as_str()).unwrap_or_default().try_into()
            }
        }

        #[automatically_derived]
        impl<'a, #generic_params> std::convert::TryFrom<Option<&'a crate::CanonicalJsonValue>> for &'a #id_ty {
            type Error = crate::IdParseError;

            fn try_from(v: Option<&'a crate::CanonicalJsonValue>) -> Result<Self, Self::Error> {
                v.and_then(|v| v.as_str()).unwrap_or_default().try_into()
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for Box<#id_ty> {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#id_ty>::parse_box(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<&str> for Box<#id_ty> {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#id_ty>::parse_box(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<String> for Box<#id_ty> {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#id_ty>::parse_box(s.into_boxed_str())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for #owned_ty {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#id_ty>::parse(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<&str> for #owned_ty {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#id_ty>::parse(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<String> for #owned_ty {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#id_ty>::parse(s)
            }
        }
    }
}

fn expand_unchecked_impls(input: &ItemStruct) -> TokenStream {
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let generic_params = &input.generics.params;

    let id = &input.ident;
    let owned = format_ident!("Owned{id}");
    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    quote! {
        #[automatically_derived]
        impl<'a, #generic_params> From<&'a str> for &'a #id_ty {
            fn from(s: &'a str) -> Self {
                <#id_ty>::from_borrowed(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&str> for #owned_ty {
            fn from(s: &str) -> Self {
                <&#id_ty>::from(s).into()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<str>> for #owned_ty {
            fn from(s: Box<str>) -> Self {
                let s: String = s.into();
                Self::from(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<String> for #owned_ty {
            fn from(s: String) -> Self {
                Self::new(s.into_bytes().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&str> for Box<#id_ty> {
            fn from(s: &str) -> Self {
                Self::from(s.to_owned().into_boxed_str())
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<str>> for Box<#id_ty> {
            fn from(s: Box<str>) -> Self {
                <#id_ty>::from_box(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<String> for Box<#id_ty> {
            fn from(s: String) -> Self {
                let s = s.into_boxed_str();
                Self::from(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<#id_ty>> for Box<str> {
            fn from(id: Box<#id_ty>) -> Self {
                <#id_ty>::into_box(id)
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for Box<#id_ty> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Box::<str>::deserialize(deserializer).map(<#id_ty>::from_box)
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for #owned_ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                // FIXME: Deserialize inner, convert that
                Box::<str>::deserialize(deserializer).map(<#id_ty>::from_box).map(Into::into)
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for &'de #id_ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <&'de str>::deserialize(deserializer).map(<#id_ty>::from_borrowed).map(Into::into)
            }
        }
    }
}

fn expand_as_str_impls(ty: TokenStream, impl_generics: &ImplGenerics<'_>) -> TokenStream {
    let partial_eq_string = expand_partial_eq_string(ty.clone(), impl_generics);

    quote! {
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Debug for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        #[automatically_derived]
        impl #impl_generics serde::Serialize for #ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #partial_eq_string
    }
}

fn expand_partial_eq_string(ty: TokenStream, impl_generics: &ImplGenerics<'_>) -> TokenStream {
    IntoIterator::into_iter([
        (ty.clone(), quote! { str }),
        (ty.clone(), quote! { &str }),
        (ty.clone(), quote! { String }),
        (quote! { str }, ty.clone()),
        (quote! { &str }, ty.clone()),
        (quote! { String }, ty),
    ])
    .map(|(lhs, rhs)| {
        quote! {
            #[automatically_derived]
            impl #impl_generics PartialEq<#rhs> for #lhs {
                fn eq(&self, other: &#rhs) -> bool {
                    AsRef::<str>::as_ref(self) == AsRef::<str>::as_ref(other)
                }
            }
        }
    })
    .collect()
}

mod kw {
    syn::custom_keyword!(validate);
    syn::custom_keyword!(inline_bytes);
}

#[derive(Default)]
struct IdDstMeta {
    validate: Option<Path>,
    inline_bytes: Option<usize>,
}

impl IdDstMeta {
    fn merge(self, other: IdDstMeta) -> syn::Result<Self> {
        let validate = match (self.validate, other.validate) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(a), Some(b)) => {
                let mut error = syn::Error::new_spanned(b, "duplicate attribute argument");
                error.combine(syn::Error::new_spanned(a, "note: first one here"));
                return Err(error);
            }
        };

        let inline_bytes = match (self.inline_bytes, other.inline_bytes) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(a), Some(b)) => {
                let mut error = syn::Error::new_spanned(b, "duplicate attribute argument");
                error.combine(syn::Error::new_spanned(a, "note: first one here"));
                return Err(error);
            }
        };

        Ok(Self { validate, inline_bytes })
    }
}

impl Parse for IdDstMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::validate = input.parse()?;
        let _: Token![=] = input.parse()?;
        let validate = Some(input.parse()?);

        let _: Option<Token![,]> = input.parse()?;

        let _: Option<kw::inline_bytes> = input.parse()?;
        let _: Option<Token![=]> = input.parse()?;
        let inline_bytes: Option<syn::LitInt> = input.parse()?;
        let inline_bytes =
            inline_bytes.map(|ib| ib.base10_digits().parse().expect("inline_bytes is an integer"));

        Ok(Self { validate, inline_bytes })
    }
}
