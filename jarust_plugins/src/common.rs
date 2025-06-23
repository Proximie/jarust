use serde::Deserialize;
use serde::Serialize;

impl_tryfrom_serde_value!(JanusId);

/// Mountpoints, Rooms and Participants Identifier.
///
/// Identifier should be by default unsigned integer, unless configured otherwise in the plugin config.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JanusId {
    /// String Identifier
    String(String),
    /// Unsigned Integer Identifier
    Uint(U63),
}

#[cfg(feature = "ffi-compatible")]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct U63 {
    // janus-mobile-sdk depends on public visibility of this field
    // https://github.com/Proximie/janus-mobile-sdk/blob/master/rslib/src/plugins/common.rs
    // Visiblity can be revisited later.
    // Prefer using the From trait to convert U63 into a u64 value if possible.
    pub inner: u64,
}

#[cfg(not(feature = "ffi-compatible"))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct U63(u64);

impl U63 {
    pub const MAX: u64 = (1 << 63) - 1;

    pub fn new(value: u64) -> Self {
        Self::new_wrapping(value)
    }

    #[cfg(feature = "ffi-compatible")]
    pub fn new_wrapping(value: u64) -> Self {
        Self {
            inner: value & U63::MAX,
        }
    }

    #[cfg(feature = "ffi-compatible")]
    pub fn new_saturating(value: u64) -> Self {
        if value > U63::MAX {
            Self { inner: U63::MAX }
        } else {
            Self { inner: value }
        }
    }

    #[cfg(not(feature = "ffi-compatible"))]
    pub fn new_wrapping(value: u64) -> Self {
        Self(value & U63::MAX)
    }

    #[cfg(not(feature = "ffi-compatible"))]
    pub fn new_saturating(value: u64) -> Self {
        if value > U63::MAX {
            Self(U63::MAX)
        } else {
            Self(value)
        }
    }
}

impl TryFrom<u64> for U63 {
    type Error = &'static str;

    #[cfg(feature = "ffi-compatible")]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > U63::MAX {
            Err("U63 only accepts values lower than 2^63 - 1")
        } else {
            Ok(Self { inner : value })
        }
    }

    #[cfg(not(feature = "ffi-compatible"))]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > U63::MAX {
            Err("U63 only accepts values lower than 2^63 - 1")
        } else {
            Ok(Self(value))
        }
    }

}

impl From<U63> for u64 {
    #[cfg(feature = "ffi-compatible")]
    fn from(value: U63) -> Self {
        value.inner
    }

    #[cfg(not(feature = "ffi-compatible"))]
    fn from(value: U63) -> Self {
        value.0
    }
}

impl Serialize for U63 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        u64::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for U63 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Ok(U63::new(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::U63;

    #[test]
    fn test_u63_conversion_with_u64() {
        assert_eq!(u64::from(U63::try_from(123_456u64).unwrap()), 123_456u64);
        assert_eq!(u64::from(U63::try_from(U63::MAX).unwrap()), U63::MAX);
        assert!(U63::try_from(U63::MAX + 1).is_err());
    }
}
