//! Unix timestamps.

use crate::{Error, Result};
use core::fmt;
use core::fmt::Formatter;
use encoding::{Decode, Encode, Reader, Writer};

#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Unix timestamps as used in OpenSSH certificates.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub(super) struct UnixTime {
    /// Number of seconds since the Unix epoch
    secs: u64,

    /// System time corresponding to this Unix timestamp
    #[cfg(feature = "std")]
    time: SystemTime,
}

impl UnixTime {
    /// Create a new Unix timestamp.
    ///
    /// `secs` is the number of seconds since the Unix epoch.
    #[cfg(not(feature = "std"))]
    pub fn new(secs: u64) -> Result<Self> {
        Ok(Self { secs })
    }

    /// Create a new Unix timestamp.
    ///
    /// This version requires `std` and caches a `SystemTime` representation.
    /// If the value is not representable, it is saturated to the maximum
    /// representable `SystemTime`.
    #[cfg(feature = "std")]
    pub fn new(secs: u64) -> Result<Self> {
        let time = system_time_from_secs(secs);
        Ok(Self { secs, time })
    }

    /// Get the current time as a Unix timestamp.
    #[cfg(feature = "std")]
    pub fn now() -> Result<Self> {
        SystemTime::now().try_into()
    }

}

#[cfg(feature = "std")]
fn system_time_from_secs(secs: u64) -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::from_secs(secs))
        .unwrap_or_else(|| {
            UNIX_EPOCH
                .checked_add(Duration::from_secs(i64::MAX as u64))
                .expect("i64::MAX secs must be representable")
        })
}

impl Decode for UnixTime {
    type Error = Error;

    fn decode(reader: &mut impl Reader) -> Result<Self> {
        u64::decode(reader)?.try_into()
    }
}

impl Encode for UnixTime {
    fn encoded_len(&self) -> encoding::Result<usize> {
        self.secs.encoded_len()
    }

    fn encode(&self, writer: &mut impl Writer) -> encoding::Result<()> {
        self.secs.encode(writer)?;
        Ok(())
    }
}

impl From<UnixTime> for u64 {
    fn from(unix_time: UnixTime) -> u64 {
        unix_time.secs
    }
}

#[cfg(feature = "std")]
impl From<UnixTime> for SystemTime {
    fn from(unix_time: UnixTime) -> SystemTime {
        unix_time.time
    }
}

impl TryFrom<u64> for UnixTime {
    type Error = Error;

    fn try_from(unix_secs: u64) -> Result<UnixTime> {
        Self::new(unix_secs)
    }
}

#[cfg(feature = "std")]
impl TryFrom<SystemTime> for UnixTime {
    type Error = Error;

    fn try_from(time: SystemTime) -> Result<UnixTime> {
        Self::new(time.duration_since(UNIX_EPOCH)?.as_secs())
    }
}

impl fmt::Debug for UnixTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.secs)
    }
}

#[cfg(test)]
mod tests {
    use super::{UnixTime};

    #[test]
    fn new_with_max_secs() {
        assert!(UnixTime::new(u64::MAX).is_ok());
    }
}
