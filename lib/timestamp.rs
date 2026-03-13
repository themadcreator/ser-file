use binrw::binrw;
use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use std::{ops::AddAssign, time::Duration};

/// An integer representing the number of 100 nanosecond periods from the Common
/// Era epoch Jan 1, 0001.
///
/// A value of `0` indicates an "invalid" or "unused" timestamp.
#[binrw]
#[br(map(u64::into))]
#[bw(map(u64::from))]
#[derive(Clone)]
pub enum Timestamp {
    Valid(u64),
    Invalid(u64),
}

impl Timestamp {
    pub const MASK: u64 = !(0b11 << 62);

    pub fn is_valid(&self) -> bool {
        matches!(self, Timestamp::Valid(_))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp::Invalid(0)
    }
}

impl std::fmt::Display for &Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match NaiveDateTime::try_from(*self) {
            Ok(naive) => naive.fmt(f),
            Err(_) => f.write_str("None"),
        }
    }
}

impl TryFrom<&Timestamp> for NaiveDateTime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &Timestamp) -> Result<Self, Self::Error> {
        match value {
            Timestamp::Valid(t) => {
                let mut naive = NaiveDate::from_ymd_opt(1, 1, 1)
                    .ok_or("Naive date failed")?
                    .and_hms_opt(0, 0, 0)
                    .ok_or("Naive time failed")?;

                let micros = (t & Timestamp::MASK) / 10;
                naive.add_assign(TimeDelta::from_std(Duration::from_micros(micros))?);
                Ok(naive)
            }
            Timestamp::Invalid(_) => Err("Cannot convert invalid timestamp".into()),
        }
    }
}

impl TryFrom<&Timestamp> for std::time::SystemTime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &Timestamp) -> Result<Self, Self::Error> {
        let naive: NaiveDateTime = value.try_into()?;
        let utc = naive.and_utc();
        let nanos_since_epoch = utc.timestamp_nanos_opt().ok_or("Timestamp out of range")?;
        if nanos_since_epoch >= 0 {
            let duration = std::time::Duration::from_nanos(nanos_since_epoch as u64);
            Ok(std::time::UNIX_EPOCH + duration)
        } else {
            let duration = std::time::Duration::from_nanos(nanos_since_epoch.abs() as u64);
            Ok(std::time::UNIX_EPOCH - duration)
        }
    }
}

impl From<u64> for Timestamp {
    fn from(value: u64) -> Self {
        match value {
            v if (v & Timestamp::MASK) == 0 => Timestamp::Invalid(v),
            v => Timestamp::Valid(v),
        }
    }
}

impl From<&Timestamp> for u64 {
    fn from(value: &Timestamp) -> Self {
        match value {
            Timestamp::Valid(v) => *v,
            Timestamp::Invalid(v) => *v,
        }
    }
}
