use pgrx::prelude::*;

pgrx::pg_module_magic!();

struct Converter<T>(T);

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
        uuid::timestamp::Timestamp::from_unix(uuid::timestamp::context::NoContext, epoch, nanoseconds)
    }
}

impl From<Converter<uuid::timestamp::Timestamp>> for pgrx::Timestamp {
    fn from(w: Converter<uuid::timestamp::Timestamp>) -> Self {
        let ts = w.0;
        let (epoch, nanoseconds) = ts.to_unix();
        pgrx::datum::Timestamp::from((epoch * 1_000_000 + nanoseconds as u64 % 1_000) as i64)
    }
}

#[pg_extern]
fn uuid_generate_v7_now() -> pgrx::Uuid {
    Converter(uuid::Uuid::now_v7()).into()
}

#[pg_extern]
fn uuid_generate_v7(ts: Timestamp) -> pgrx::Uuid {
    let u = uuid::Uuid::new_v7(Converter(ts).into());
    Converter(u).into()
}

#[pg_extern]
fn uuid_get_timestamp(uuid: pgrx::Uuid) -> Timestamp {
    let u: uuid::Uuid = Converter(uuid).into();
    let ts = u.get_timestamp().unwrap();
    Converter(ts).into()
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_pgx_uuidv7_now() {
        let g = uuid_generate_v7_now();
        let u = uuid::Uuid::from_slice(g.as_bytes()).unwrap();
        assert_eq!(7, u.get_version_num());
    }

    #[pg_test]
    fn test_pgx_uuidv7() {
        let ts: Timestamp =
            <Timestamp as std::str::FromStr>::from_str("2021-01-01 00:00:00.0").unwrap();
        let g = uuid_generate_v7(ts);
        let u = uuid::Uuid::from_slice(g.as_bytes()).unwrap();
        assert_eq!(7, u.get_version_num());
    }

    #[pg_test]
    fn test_pgx_uuidv7_get_timestamp() {
        let it: Timestamp =
            <Timestamp as std::str::FromStr>::from_str("2021-01-01 12:34:56.789").unwrap();
        let g = uuid_generate_v7(it);
        let ot = uuid_get_timestamp(g);
        assert_eq!(it.to_iso_string(), ot.to_iso_string());
        assert_eq!(it, ot);
    }

}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
