use pgrx::prelude::*;

pub(crate) struct Converter<T>(pub T);

impl From<Converter<uuid::Uuid>> for pgrx::Uuid {
    fn from(w: Converter<uuid::Uuid>) -> Self {
        pgrx::Uuid::from_bytes(*w.0.as_bytes())
    }
}

impl From<Converter<pgrx::Uuid>> for uuid::Uuid {
    fn from(w: Converter<pgrx::Uuid>) -> Self {
        uuid::Uuid::from_bytes(*w.0.as_bytes())
    }
}

impl From<Converter<pgrx::Timestamp>> for uuid::timestamp::Timestamp {
    fn from(w: Converter<pgrx::Timestamp>) -> Self {
        let ts = w.0;
        let epoch: u64 = ts
            .extract_part(DateTimeParts::Epoch)
            .unwrap()
            .try_into()
            .unwrap();
        let nanoseconds: u32 = (ts.second().fract() * 1_000_000_000.0) as u32;
        uuid::timestamp::Timestamp::from_unix(
            uuid::timestamp::context::NoContext,
            epoch,
            nanoseconds,
        )
    }
}

impl From<Converter<uuid::timestamp::Timestamp>> for pgrx::Timestamp {
    fn from(w: Converter<uuid::timestamp::Timestamp>) -> Self {
        let ts = w.0;
        let (epoch, nanoseconds) = ts.to_unix();
        pgrx::datum::Timestamp::from((epoch * 1_000_000 + nanoseconds as u64 % 1_000) as i64)
    }
}
