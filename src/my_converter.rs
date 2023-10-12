use pgrx::prelude::*;

pub(crate) struct Converter<T>(pub T);

impl<T> Converter<T> {
    fn unwrap(self) -> T {
        self.0
    }
}

impl From<Converter<uuid::Uuid>> for pgrx::Uuid {
    fn from(w: Converter<uuid::Uuid>) -> Self {
        pgrx::Uuid::from_bytes(*w.unwrap().as_bytes())
    }
}

impl From<Converter<pgrx::Uuid>> for uuid::Uuid {
    fn from(w: Converter<pgrx::Uuid>) -> Self {
        uuid::Uuid::from_bytes(*w.unwrap().as_bytes())
    }
}

impl From<Converter<pgrx::Timestamp>> for uuid::timestamp::Timestamp {
    fn from(w: Converter<pgrx::Timestamp>) -> Self {
        let ts = w.unwrap();
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
        let ts = w.unwrap();
        let (epoch, nanoseconds) = ts.to_unix();
        pgrx::datum::Timestamp::from((epoch * 1_000_000 + nanoseconds as u64 / 1_000) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid() {
        let u: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000000");
        let p: pgrx::Uuid = Converter(u).into();
        let u2: uuid::Uuid = Converter(p).into();
        assert_eq!(u, u2);
    }

    #[test]
    fn timestamp() {
        let t: uuid::timestamp::Timestamp =
            uuid::timestamp::Timestamp::now(uuid::timestamp::context::NoContext);
        let p: pgrx::Timestamp = Converter(t).into();
        let t2: uuid::timestamp::Timestamp = Converter(p).into();
        assert_eq!(t, t2);
    }
}
