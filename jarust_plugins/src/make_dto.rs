/// DTO creation macro for jarust_plugins
///
/// [`make_dto!`] takes a name for the struct, a list of required fields and a list of optional fields
/// and create the DTO struct, derive useful traits and implements [`TryFrom`] for [`serde_json::Value`] using [`impl_tryfrom_serde_value!`]
/// so we can easily convert a [`serde_json::Value`] to the struct by using `let message: serde_json::Value = dto.try_into().unwrap();`
///
/// The macro behaves differently depending on the number of required and optional fields:
///
/// - A struct with a single required field and a single optional field will keep the required and the optional at the top level struct
/// - A struct with a single required field but multiple optional fields will keep the required at the top level
///   but will create a separate struct for the optional fields
/// - A struct with multiple required fields and a single optional field will keep the required fields and the optional
///   field at the top level
/// - A struct with multiple required fields and multiple optional fields will create separate struct for the required fields
///   and a separate struct for the optional fields
///
/// ## Example
///
/// ```rust
/// make_dto!(User, required { id: u64, name: String}, optional { nickname: String, job_position: String });
/// ```
/// This will expand to:
///
/// ```rust
/// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
/// pub struct UserRequired {
///     id: u64,
///     name: String
/// }
///
/// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, serde::Serialize)]
/// pub struct UserOptional {
///     #[serde(skip_serializing_if = "Option::is_none")]
///     nickname: Option<String>,
///     #[serde(skip_serializing_if = "Option::is_none")]
///     job_position: Option<String>
/// }
///
/// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
/// pub struct User {
///     #[serde(flatten)]
///     required: UserRequired,
///     #[serde(flatten)]
///     optional: UserOptional
/// }
///
/// impl_tryfrom_serde_value!(User UserOptional UserRequired);
/// ```
macro_rules! make_dto {
    (
        $(#[$main_attr:meta])* $main:ident,
        required { $(#[$rfield_attr:meta])* $rfield:ident: $rtype:ty $(,)? } $(,)?
    ) => {
        $(#[$main_attr])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
        pub struct $main {
            $(#[$rfield_attr])*
            pub $rfield: $rtype
        }

        impl_tryfrom_serde_value!($main);
    };

    (
        $(#[$main_attr:meta])* $main:ident,
        optional { $(#[$ofield_attr:meta])* $ofield:ident: $otype:ty $(,)? } $(,)?
    ) => {
        $(#[$main_attr])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
        pub struct $main {
            $(#[$ofield_attr])*
            #[serde(skip_serializing_if = "Option::is_none")]
            pub $ofield: Option<$otype>
        }

        impl_tryfrom_serde_value!($main);
    };

    // Arguably overkill, we can pass 2 params to the function instead of constructing a DTO
    // But can be useful for simple named params
    (
        $(#[$main_attr:meta])* $main:ident,
        required { $(#[$rfield_attr:meta])* $rfield:ident: $rtype:ty $(,)? },
        optional { $(#[$ofield_attr:meta])* $ofield:ident: $otype:ty $(,)? }
    ) => {
        $(#[$main_attr])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
        pub struct $main {
            $(#[$rfield_attr])*
            pub $rfield: $rtype,
            $(#[$ofield_attr])*
            #[serde(skip_serializing_if = "Option::is_none")]
            pub $ofield: Option<$otype>,
        }

        impl_tryfrom_serde_value!($main);
    };

    (
        $(#[$main_attr:meta])* $main:ident,
        required { $(#[$rfield_attr:meta])* $rfield:ident: $rtype:ty $(,)? },
        optional { $( $(#[$ofield_attr:meta])* $ofield:ident: $otype:ty ),* $(,)? }
    ) => {
        paste::paste! {
            $(#[$main_attr])*
            #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, serde::Serialize)]
            pub struct [<$main Optional>] {
                $(
                    $(#[$ofield_attr])*
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub $ofield: Option<$otype>,
                )*
            }

            $(#[$main_attr])*
            #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
            pub struct $main {
                $(#[$rfield_attr])*
                pub $rfield: $rtype,
                #[serde(flatten)]
                pub optional: [<$main Optional>],
            }

            impl_tryfrom_serde_value!($main [<$main Optional>]);
        }
    };

    (
        $(#[$main_attr:meta])* $main:ident,
        required { $( $(#[$rfield_attr:meta])* $rfield:ident: $rtype:ty ),* $(,)? },
        optional { $(#[$ofield_attr:meta])* $ofield:ident: $otype:ty $(,)? }
    ) => {
        paste::paste! {
            $(#[$main_attr])*
            #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
            pub struct $main {
                $(
                    $(#[$rfield_attr])*
                    pub $rfield: $rtype,
                )*
                $(#[$ofield_attr])*
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $ofield: Option<$otype>,
            }

            impl_tryfrom_serde_value!($main);
        }
    };

    (
        $(#[$main_attr:meta])* $main:ident,
        required { $( $(#[$rfield_attr:meta])* $rfield:ident: $rtype:ty ),* $(,)? },
        optional { $( $(#[$ofield_attr:meta])* $ofield:ident: $otype:ty ),* $(,)? }
    ) => {
        paste::paste! {
            $(#[$main_attr])*
            #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
            pub struct [<$main Required>] {
                $(
                    $(#[$rfield_attr])*
                    pub $rfield: $rtype,
                )*
            }

            $(#[$main_attr])*
            #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, serde::Serialize)]
            pub struct [<$main Optional>] {
                $(
                    $(#[$ofield_attr])*
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub $ofield: Option<$otype>,
                )*
            }

            $(#[$main_attr])*
            #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
            pub struct $main {
                #[serde(flatten)]
                pub required: [<$main Required>],
                #[serde(flatten)]
                pub optional: [<$main Optional>],
            }

            impl_tryfrom_serde_value!($main [<$main Optional>] [<$main Required>]);
        }
    };

    (
        $(#[$main_attr:meta])* $main:ident,
        required { $( $(#[$rfield_attr:meta])* $rfield:ident: $rtype:ty ),* $(,)? } $(,)?
    ) => {
        $(#[$main_attr])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, serde::Serialize)]
        pub struct $main {
            $(
                $(#[$rfield_attr])*
                pub $rfield: $rtype,
            )*
        }

        impl_tryfrom_serde_value!($main);
    };

    (
        $(#[$main_attr:meta])* $main:ident,
        optional { $( $(#[$ofield_attr:meta])* $ofield:ident: $otype:ty ),* $(,)? } $(,)?
    ) => {
        $(#[$main_attr])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, serde::Serialize)]
        pub struct $main {
            $(
                $(#[$ofield_attr])*
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $ofield: Option<$otype>,
            )*
        }

        impl_tryfrom_serde_value!($main);
    };
}
