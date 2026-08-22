//! Functions to generate the `*EventType` enums.

use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{EventEnumData, EventEnumKind};
use crate::util::{RumaEvents, RumaEventsReexport};

/// Data to generate an `*EventType` enum.
pub(super) struct EventTypeEnum<'a> {
    /// The data for the enum.
    data: &'a EventEnumData,

    /// The import path for the ruma-events crate.
    ruma_events: &'a RumaEvents,

    /// The import path for the serde crate.
    serde: TokenStream,

    /// The name of the event type enum
    ident: syn::Ident,
}

impl<'a> EventTypeEnum<'a> {
    /// Create an `EventTypeEnum` with the given data.
    pub(super) fn new(data: &'a EventEnumData, ruma_events: &'a RumaEvents) -> Self {
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);

        let ident = data.kind.to_event_type_enum();

        Self { data, ruma_events, ident, serde }
    }
}

impl EventTypeEnum<'_> {
    /// Generate the `*EventType` enum and its implementations.
    pub(super) fn expand(&self) -> TokenStream {
        let ident = &self.ident;
        let ruma_events = self.ruma_events;
        let enum_doc = format!("The type of `{}` this is.", self.kind);

        let variants = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let variant_docs = event.docs();

            if event.has_type_fragment() {
                quote! {
                    #variant_docs
                    #( #variant_attrs )*
                    #variant(#ruma_events::EventTypeString),
                }
            } else {
                quote! {
                    #variant_docs
                    #( #variant_attrs )*
                    #variant,
                }
            }
        });

        let ord_impl = self.expand_ord_impl();
        let to_string_impl = self.expand_to_string_impl();
        let from_string_impl = self.expand_from_string_impl();
        let into_timeline_event_type_impl = self.expand_into_timeline_event_type_impl();

        quote! {
            #[doc = #enum_doc]
            ///
            /// This type can hold an arbitrary string. To build events with a custom type, convert it
            /// from a string with `::from()` / `.into()`. To check for events that are not available as a
            /// documented variant here, use its string representation, obtained through `.to_string()`.
            #[cfg_attr(feature = "unstable-uniffi", derive(uniffi::Enum))]
            #[cfg_attr(feature = "unstable-uniffi", uniffi::export(Display, Eq, Hash))]
            #[derive(Clone, PartialEq, Eq, Hash)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            pub enum #ident {
                #( #variants )*
                #[doc(hidden)]
                /// This variant ensures forward compatibility of the library. It deliberately cannot be
                /// used to create custom variants in client code.
                _Custom(crate::PrivOwnedSmallStr),
            }

            #ord_impl
            #to_string_impl
            #from_string_impl
            #into_timeline_event_type_impl
        }
    }

    /// Generate the `Ord` and `PartialOrd` implementations for the event type enum.
    ///
    /// To compare event types we need to compare the static event type first, and then the "type
    /// fragment" if there is one.
    fn expand_ord_impl(&self) -> TokenStream {
        let ident = &self.ident;

        let event_type_str_match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let ev_type = &event.types.ev_type;

            if ev_type.is_prefix() {
                let ev_type = ev_type.without_wildcard();
                quote! {
                    #( #variant_attrs )*
                    Self::#variant(_s) => #ev_type,
                }
            } else {
                quote! {
                    #( #variant_attrs )*
                    Self::#variant => #ev_type,
                }
            }
        });

        let mut type_fragment_match_arms = self
            .events
            .iter()
            // We only need to compare types with fragment, others will be equal.
            .filter(|event| event.has_type_fragment())
            .map(|event| {
                let variant = &event.ident;
                let variant_attrs = &event.attrs;

                quote! {
                    #( #variant_attrs )*
                    (Self::#variant(this), Self::#variant(other)) => this.cmp(other),
                }
            })
            .peekable();

        let cmp_type_fragment_impl = if type_fragment_match_arms.peek().is_none() {
            // If there are no type fragments, all variants are equal.
            quote! { ::std::cmp::Ordering::Equal }
        } else {
            quote! {
                match (self, other) {
                    #( #type_fragment_match_arms )*
                    _ => ::std::cmp::Ordering::Equal,
                }
            }
        };

        quote! {
            #[allow(deprecated)]
            impl #ident {
                fn event_type_str(&self) -> &::std::primitive::str {
                    match self {
                        #( #event_type_str_match_arms )*
                        Self::_Custom(crate::PrivOwnedSmallStr(s)) => s,
                    }
                }

                fn cmp_type_fragment(&self, other: &Self) -> ::std::cmp::Ordering {
                    #cmp_type_fragment_impl
                }
            }

            impl ::std::cmp::Ord for #ident {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    let event_type_cmp = self.event_type_str().cmp(&other.event_type_str());

                    if event_type_cmp.is_eq() {
                        self.cmp_type_fragment(other)
                    } else {
                        event_type_cmp
                    }
                }
            }

            impl ::std::cmp::PartialOrd for #ident {
                fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
        }
    }

    /// Generate the `std::fmt::Display`, `std::fmt::Debug` and `serde::Serialize` implementations
    /// for the event type enum.
    fn expand_to_string_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let serde = &self.serde;

        let match_arms = self.events
            .iter()
            .map(|event| {
                let variant = &event.ident;
                let variant_attrs = &event.attrs;
                let ev_type = &event.types.ev_type;

                if ev_type.is_prefix() {
                    let format_str = ev_type.without_wildcard().to_owned() + "{}";
                    quote! {
                        #( #variant_attrs )*
                        Self::#variant(_s) => ::std::borrow::Cow::Owned(::std::format!(#format_str, _s)),
                    }
                } else {
                    quote! {
                        #( #variant_attrs )*
                        Self::#variant => ::std::borrow::Cow::Borrowed(#ev_type),
                    }
                }
            });

        // `Serialize` writes a type with a fragment through `Display`, so it does not take the
        // allocation `to_cow_str` needs to hand back one string slice.
        let serialize_match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let ev_type = &event.types.ev_type;

            if ev_type.is_prefix() {
                quote! {
                    #( #variant_attrs )*
                    Self::#variant(_) => serializer.collect_str(self),
                }
            } else {
                quote! {
                    #( #variant_attrs )*
                    Self::#variant => serializer.serialize_str(#ev_type),
                }
            }
        });

        // `Display` writes the two pieces of a type with a fragment one after the other, so it
        // does not take the allocation `to_cow_str` needs to hand back one string slice.
        let display_match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let ev_type = &event.types.ev_type;

            if ev_type.is_prefix() {
                let prefix = ev_type.without_wildcard();

                quote! {
                    #( #variant_attrs )*
                    Self::#variant(fragment) => {
                        f.write_str(#prefix)?;
                        f.write_str(fragment)
                    }
                }
            } else {
                quote! {
                    #( #variant_attrs )*
                    Self::#variant => f.write_str(#ev_type),
                }
            }
        });

        quote! {
            #[allow(deprecated)]
            impl #ident {
                /// Access the string for the type
                pub fn to_cow_str(&self) -> ::std::borrow::Cow<'_, ::std::primitive::str> {
                    match self {
                        #( #match_arms )*
                        Self::_Custom(crate::PrivOwnedSmallStr(s)) => ::std::borrow::Cow::Borrowed(s),
                    }
                }
            }

            #[allow(deprecated)]
            impl ::std::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    // Padding and truncation need the whole type as one slice, which a type with
                    // a fragment only has once it is joined.
                    if f.width().is_some() || f.precision().is_some() {
                        return f.pad(&self.to_cow_str());
                    }

                    match self {
                        #( #display_match_arms )*
                        Self::_Custom(crate::PrivOwnedSmallStr(s)) => f.write_str(s),
                    }
                }
            }

            #[allow(deprecated)]
            impl ::std::fmt::Debug for #ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    <str as ::std::fmt::Debug>::fmt(&self.to_cow_str(), f)
                }
            }

            #[allow(deprecated)]
            impl #serde::Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: #serde::Serializer,
                {
                    match self {
                        #( #serialize_match_arms )*
                        Self::_Custom(crate::PrivOwnedSmallStr(s)) => serializer.serialize_str(s),
                    }
                }
            }
        }
    }

    /// Generate the `from_known_str` helper and the `From<&str>`, `From<String>` and
    /// `serde::Deserialize` implementations for the event type enum.
    fn expand_from_string_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let ruma_common = self.ruma_events.ruma_common();
        let serde = &self.serde;

        let from_str_match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let ev_types = event.types.iter();

            if event.has_type_fragment() {
                ev_types.map(|ev_type| {
                    let prefix = ev_type.without_wildcard();

                    quote! {
                        #( #variant_attrs )*
                        // Use if-let guard once available
                        s if s.starts_with(#prefix) => {
                            Self::#variant(::std::convert::From::from(s.strip_prefix(#prefix).unwrap()))
                        }
                    }
                }).collect()
            } else {
                quote! {
                    #( #variant_attrs )*
                    #( #ev_types )|* => Self::#variant,
                }
            }
        });

        fn to_snake_case(ident: String) -> String {
            let mut s = String::with_capacity(ident.len());

            for (i, ch) in ident.char_indices() {
                if i > 0 && ch.is_uppercase() {
                    s.push('_');
                }

                s.push(ch.to_ascii_lowercase());
            }

            s
        }

        let ident_from_str = format_ident!("{}_from_string", to_snake_case(ident.to_string()));

        quote! {
            #[allow(deprecated)]
            impl #ident {
                /// Match one of the documented event types, without taking ownership of `s`.
                ///
                /// Returns `None` for a type that only the `_Custom` variant can hold, which lets
                /// a caller hand its own allocation to that variant instead of copying the string
                /// and dropping the original.
                fn from_known_str(s: &::std::primitive::str) -> Option<Self> {
                    Some(match s {
                        #( #from_str_match_arms )*
                        _ => return None,
                    })
                }
            }

            #[allow(deprecated)]
            impl ::std::convert::From<&::std::primitive::str> for #ident {
                fn from(s: &::std::primitive::str) -> Self {
                    Self::from_known_str(s).unwrap_or_else(|| {
                        Self::_Custom(crate::PrivOwnedSmallStr(::std::convert::From::from(s)))
                    })
                }
            }

            #[allow(deprecated)]
            impl ::std::convert::From<::std::string::String> for #ident {
                fn from(s: ::std::string::String) -> Self {
                    Self::from_known_str(&s).unwrap_or_else(|| {
                        Self::_Custom(crate::PrivOwnedSmallStr::from_string(s))
                    })
                }
            }

            #[allow(deprecated)]
            impl<'a> ::std::convert::From<::std::borrow::Cow<'a, ::std::primitive::str>> for #ident {
                fn from(s: ::std::borrow::Cow<'a, ::std::primitive::str>) -> Self {
                    match s {
                        ::std::borrow::Cow::Borrowed(s) => Self::from(s),
                        ::std::borrow::Cow::Owned(s) => Self::from(s),
                    }
                }
            }

            #[cfg(feature = "unstable-uniffi")]
            #[uniffi::export]
            /// Construct a variant of the enum from a string.
            fn #ident_from_str(s: ::std::string::String) -> #ident {
                s.into()
            }

            #[allow(deprecated)]
            impl<'de> #serde::Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>
                {
                    #ruma_common::serde::deserialize_cow_str(deserializer)
                        .map(::std::convert::From::from)
                }
            }
        }
    }

    /// Generate the `From<{ident}> for TimelineEventType` implementation for the timeline kinds.
    fn expand_into_timeline_event_type_impl(&self) -> Option<TokenStream> {
        if !self.kind.is_timeline() || self.kind == EventEnumKind::Timeline {
            return None;
        }

        let ident = &self.ident;

        let match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;

            if event.has_type_fragment() {
                quote! {
                    #( #variant_attrs )*
                    #ident::#variant(s) => Self::#variant(s),
                }
            } else {
                quote! {
                    #( #variant_attrs )*
                    #ident::#variant => Self::#variant,
                }
            }
        });

        Some(quote! {
            #[allow(deprecated)]
            impl ::std::convert::From<#ident> for TimelineEventType {
                fn from(s: #ident) -> Self {
                    match s {
                        #( #match_arms )*
                        #ident ::_Custom(_s) => Self::_Custom(_s),
                    }
                }
            }
        })
    }
}

impl Deref for EventTypeEnum<'_> {
    type Target = EventEnumData;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
