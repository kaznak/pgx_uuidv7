use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use pgrx::prelude::*;

#[derive(Debug)]
pub(crate) struct Converter<T>(pub T);

impl<T> Converter<T> {
    fn unwrap(self) -> T {
        self.0
    }
}

impl From<Converter<uuid::Uuid>> for pgrx::Uuid {
    #[inline]
    fn from(w: Converter<uuid::Uuid>) -> Self {
        pgrx::Uuid::from_bytes(*w.unwrap().as_bytes())
    }
}

impl From<Converter<pgrx::Uuid>> for uuid::Uuid {
    #[inline]
    fn from(w: Converter<pgrx::Uuid>) -> Self {
        uuid::Uuid::from_bytes(*w.unwrap().as_bytes())
    }
}

impl From<Converter<pgrx::TimestampWithTimeZone>> for uuid::Timestamp {
    #[inline]
    fn from(w: Converter<pgrx::TimestampWithTimeZone>) -> Self {
        let ts = w.unwrap().to_utc();
        let epoch: u64 = ts
            .extract_part(DateTimeParts::Epoch)
            .unwrap()
            .try_into()
            .unwrap();
        let nanoseconds: u32 = (ts.second().fract() * 1_000_000_000.0) as u32;
        uuid::Timestamp::from_unix(uuid::timestamp::context::NoContext, epoch, nanoseconds)
    }
}

impl From<Converter<uuid::Timestamp>> for chrono::DateTime<Utc> {
    #[inline]
    fn from(value: Converter<uuid::Timestamp>) -> Self {
        let ts = value.unwrap();
        let (epoch, nanoseconds) = ts.to_unix();
        let naive_datetime = NaiveDateTime::from_timestamp_opt(epoch as i64, nanoseconds).unwrap();
        DateTime::from_naive_utc_and_offset(naive_datetime, Utc)
    }
}

impl From<Converter<chrono::DateTime<Utc>>> for pgrx::TimestampWithTimeZone {
    #[inline]
    fn from(w: Converter<chrono::DateTime<Utc>>) -> Self {
        let dt = w.unwrap();
        pgrx::TimestampWithTimeZone::with_timezone(
            dt.year() as i32,
            dt.month() as u8,
            dt.day() as u8,
            dt.hour() as u8,
            dt.minute() as u8,
            dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
            "UTC",
        )
        .unwrap()
    }
}

impl From<Converter<uuid::Timestamp>> for pgrx::TimestampWithTimeZone {
    #[inline]
    fn from(w: Converter<uuid::Timestamp>) -> Self {
        // Using pgrx::datum::Timestamp::from does not work well.
        // because Postgres Epoch is from 2000-01-01 00:00:00 UTC.
        // refer: https://docs.rs/pgrx/0.10.2/pgrx/datum/struct.Timestamp.html#impl-From%3Ci64%3E-for-Timestamp
        let ts = w.unwrap();
        let datetime: DateTime<Utc> = Converter(ts).into();
        Converter(datetime).into()
    }
}

pub fn to_uuid_timestamp_buildpart(ts: pgrx::TimestampWithTimeZone) -> u64 {
    let ut: uuid::Timestamp = Converter(ts).into();
    let (secs, nanos) = ut.to_unix();
    let millis = (secs * 1000).saturating_add(nanos as u64 / 1_000_000);
    millis
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[pg_test]
    fn uuid000() {
        let u: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000000");
        let p: pgrx::Uuid = Converter(u).into();
        let u2: uuid::Uuid = Converter(p).into();
        assert_eq!(u, u2);
    }

    #[pg_test]
    fn timestamp000() {
        let dt000 = NaiveDate::from_ymd_opt(2012, 3, 4)
            .unwrap()
            .and_hms_nano_opt(5, 6, 7, 123_456_789)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        assert_eq!(
            dt000.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            "2012-03-04T05:06:07.123456789Z"
        );
        assert_eq!(dt000.timestamp(), 1_330_837_567);
        assert_eq!(dt000.timestamp_subsec_nanos(), 123_456_789);
    }

    #[pg_test]
    fn timestamp001() {
        let ut000: uuid::Timestamp = uuid::Timestamp::from_unix(
            uuid::timestamp::context::NoContext,
            1_330_837_567,
            123_456_789,
        );
        let (epoch, nanoseconds) = ut000.to_unix();
        assert_eq!(epoch, 1_330_837_567);
        assert_eq!(nanoseconds, 123_456_789);
    }

    #[pg_test]
    fn timestamp002() {
        let ut000: uuid::Timestamp = uuid::Timestamp::from_unix(
            uuid::timestamp::context::NoContext,
            1_330_837_567,
            123_456_789,
        );
        let pt000: pgrx::TimestampWithTimeZone = Converter(ut000).into();
        // rounded up to microseconds
        assert_eq!(
            pt000.to_iso_string_with_timezone("UTC").unwrap(),
            "2012-03-04T05:06:07.123457+00:00"
        );
    }

    #[pg_test]
    fn timestamp003() {
        let pt000: pgrx::TimestampWithTimeZone =
            pgrx::TimestampWithTimeZone::with_timezone(2012, 3, 4, 5, 6, 7.123456789, "UTC")
                .unwrap();
        let ut000: uuid::Timestamp = Converter(pt000).into();
        let (epoch, nanoseconds) = ut000.to_unix();
        assert_eq!(epoch, 1_330_837_567);
        // rounded up to microseconds
        assert_eq!(nanoseconds, 123_457_000);
    }
}
