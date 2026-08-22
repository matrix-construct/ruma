/// Convenience macro to declare a struct named `PrivOwnedStr`.
///
/// The generated struct is a wrapper around `Box<str>` that cannot be used in a meaningful way
/// outside of the crate where it is defined. It is usually used for string enums because their
/// `_Custom` variant can't be truly private (only `#[doc(hidden)]`).
///
/// The struct implements `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`.
///
/// ## Arguments
///
/// This macro can be called without any arguments, it will only generate the struct and its basic
/// implementations.
///
/// The following keywords can also be used as a comma-separated list to add more implementations to
/// the struct:
///
/// - `uniffi` - Expose the struct as an object named `PrivateString` to foreign languages via
///   [`uniffi`], behind an `unstable-uniffi` cargo feature. This is necessary to expose an enum or
///   record using `PrivOwnedStr` to foreign languages. Requires the crate calling the macro to have
///   an `unstable-uniffi` cargo feature and [`uniffi` must be set up][uniffi-setup].
///
/// ## Example
///
/// ```
/// ruma_common::priv_owned_str!(uniffi);
/// ```
///
/// [uniffi]: https://crates.io/crates/uniffi
/// [uniffi-setup]: https://mozilla.github.io/uniffi-rs/latest/tutorial/Rust_scaffolding.html
#[doc(hidden)]
#[macro_export]
macro_rules! priv_owned_str {
    () => {
        #[doc(hidden)]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PrivOwnedStr(std::boxed::Box<std::primitive::str>);

        impl std::fmt::Debug for PrivOwnedStr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };

    ( $( $keyword:ident ),+ ) => {
        $crate::priv_owned_str!();
        $( $crate::priv_owned_str!(@keyword $keyword); )+
    };

    ( @keyword uniffi ) => {
        // Wrapper around `Box<str>` for transferring `PrivOwnedStr` over UniFFI.
        // We cannot derive `PrivOwnedStr` from `uniffi::Object` directly because
        // that would require wrapping it in an `Arc` inside the `_Custom` variants.
        #[cfg(feature = "unstable-uniffi")]
        #[derive(uniffi::Object)]
        #[doc(hidden)]
        pub struct PrivateString(std::boxed::Box<std::primitive::str>);

        #[cfg(feature = "unstable-uniffi")]
        uniffi::custom_type!(PrivOwnedStr, std::sync::Arc<PrivateString> , {
            lower: |value| std::sync::Arc::new(PrivateString(value.0)),
            try_lift: |value| Ok(PrivOwnedStr(value.0.clone())),
        });
    };
}

/// Convenience macro to declare a struct named `PrivOwnedSmallStr`.
///
/// This serves the same purpose as [`priv_owned_str!`], but the wrapper holds a
/// [`smallstr::SmallString`] instead of a `Box<str>`, so a short value stays inline rather than
/// taking a heap allocation. The inline buffer widens every value of the enum carrying the
/// `_Custom` variant, trading stack bytes on the common documented types for an allocation
/// spared on each custom one, so size the wrapped type to the modal custom string.
///
/// The struct implements `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`.
///
/// ## Arguments
///
/// The first argument names the wrapped type, which must be a `smallstr::SmallString`. UniFFI
/// takes the name of a custom type from a plain identifier, so a type from another crate is
/// imported before it is passed here.
///
/// The `uniffi` keyword can follow it, with the same meaning as in [`priv_owned_str!`], except
/// that the value crosses the FFI boundary as a plain string rather than as an object. The
/// wrapped type gets a bridge of its own, for the generated code that holds one beside the
/// wrapper.
///
/// ## Example
///
/// ```
/// pub type MyString = smallstr::SmallString<[u8; 40]>;
/// ruma_common::priv_owned_small_str!(MyString);
/// ```
///
/// [`smallstr::SmallString`]: https://docs.rs/smallstr/latest/smallstr/struct.SmallString.html
#[doc(hidden)]
#[macro_export]
macro_rules! priv_owned_small_str {
    ( $inner:ident ) => {
        #[doc(hidden)]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PrivOwnedSmallStr($inner);

        impl PrivOwnedSmallStr {
            /// Take ownership of `s`, keeping its allocation only when the value is too long to
            /// be held inline.
            ///
            /// Deliberately not `pub`: a public constructor would let other crates build the
            /// `_Custom` variant this type exists to keep private.
            #[allow(dead_code)]
            fn from_string(s: std::string::String) -> Self {
                if s.len() > <$inner>::new().inline_size() {
                    Self(<$inner>::from_string(s))
                } else {
                    Self(<$inner>::from_str(&s))
                }
            }
        }

        impl std::fmt::Debug for PrivOwnedSmallStr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };

    ( $inner:ident, uniffi ) => {
        $crate::priv_owned_small_str!($inner);

        #[cfg(feature = "unstable-uniffi")]
        uniffi::custom_type!(PrivOwnedSmallStr, std::string::String, {
            lower: |value| value.0.into_string(),
            try_lift: |value| Ok(PrivOwnedSmallStr::from_string(value)),
        });

        // The wrapped type is foreign, so its bridge is declared `remote` and implements the
        // conversion traits for the calling crate's `UniFfiTag` alone. Generated code holding
        // the wrapped type itself beside the wrapper, as an event type enum holds one in its
        // type fragment variants, needs the bridge in the crate it is generated into.
        #[cfg(feature = "unstable-uniffi")]
        uniffi::custom_type!($inner, std::string::String, {
            remote,
            lower: |value| value.into_string(),
            try_lift: |value| Ok(PrivOwnedSmallStr::from_string(value).0),
        });
    };
}
