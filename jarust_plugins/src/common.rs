use serde::de;
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

    #[cfg(feature = "ffi-compatible")]
    fn new(value: u64) -> Self {
        Self { inner: value }
    }

    #[cfg(not(feature = "ffi-compatible"))]
    fn new(value: u64) -> Self {
        Self(value)
    }

    #[cfg(feature = "ffi-compatible")]
    fn inner(&self) -> u64 {
        self.inner
    }

    #[cfg(not(feature = "ffi-compatible"))]
    fn inner(&self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for U63 {
    type Error = std::num::TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > U63::MAX {
            // Work around impossible instanciation from outside of std
            // by generating a known to fail conversion.
            // Should compile to nothing with optimisation level other than 0.
            Err(<u8 as TryFrom<u16>>::try_from(300).unwrap_err())
        } else {
            Ok(U63::new(value))
        }
    }
}

impl From<U63> for u64 {
    fn from(value: U63) -> Self {
        value.inner()
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
        U63::try_from(value).map_err(|_| -> D::Error {
            de::Error::invalid_value(
                de::Unexpected::Unsigned(value.into()),
                &"a value less than or equal to 9223372036854775807",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::U63;

    #[test]
    fn test_u63_conversion_with_u64() {
        assert_eq!(u64::from(U63::try_from(123_456u64).unwrap()), 123_456u64);
        assert_eq!(u64::from(U63::try_from(U63::MAX).unwrap()), U63::MAX);
        assert!(U63::try_from(U63::MAX + 1).is_err());
    }

    #[test]
    fn test_u63_serialization() {
        assert_eq!(
            serde_json::to_string(&U63::try_from(123_456u64).unwrap()).unwrap(),
            "123456"
        );
        assert_eq!(
            serde_json::from_str::<U63>("123456").unwrap(),
            U63::try_from(123_456u64).unwrap()
        );

        assert_eq!(
            serde_json::to_string(&U63::try_from(U63::MAX).unwrap()).unwrap(),
            "9223372036854775807"
        );
        assert_eq!(
            serde_json::from_str::<U63>("9223372036854775807").unwrap(),
            U63::try_from(U63::MAX).unwrap()
        );

        assert_eq!(
            serde_json::from_str::<U63>("9223372036854775808").unwrap_err().to_string(),
            "invalid value: integer `9223372036854775808`, expected a value less than or equal to 9223372036854775807"
        );
    }
}
