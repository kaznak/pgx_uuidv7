mod my_converter;

use my_converter::Converter;
use pgrx::prelude::*;
use uuid::Uuid;

pgrx::pg_module_magic!();

#[pg_extern(parallel_safe)]
fn uuid_generate_v7_now() -> pgrx::Uuid {
    Converter(Uuid::now_v7()).into()
}

#[pg_extern(parallel_safe)]
fn uuid_generate_v7(ts: pgrx::Timestamp) -> pgrx::Uuid {
    let u = Uuid::new_v7(Converter(ts).into());
    Converter(u).into()
}

#[pg_extern(immutable, parallel_safe)]
fn uuid_to_timestamp(uuid: pgrx::Uuid) -> pgrx::Timestamp {
    let u: uuid::Uuid = Converter(uuid).into();
    let ts = u.get_timestamp().unwrap();
    Converter(ts).into()
}

#[pg_extern(parallel_safe)]
fn timestamp_to_uuid_random(ts: pgrx::Timestamp) -> pgrx::Uuid {
    uuid_generate_v7(ts)
}

// #[pg_extern(immutable, parallel_safe)]
fn timestamp_to_uuid(ts: pgrx::Timestamp, rv: u32) -> pgrx::Uuid {
    let ut: uuid::Timestamp = Converter(ts).into();
    let (secs, nanos) = ut.to_unix();
    let millis = (secs * 1000).saturating_add(nanos as u64 / 1_000_000);
    let u: uuid::Uuid = uuid::Builder::from_unix_timestamp_millis(millis, rv.to_be_bytes()[..10].try_into().unwrap()).into_uuid();
    Converter(u).into()
}

#[pg_extern(immutable, parallel_safe)]
fn timestamp_to_uuid_min(ts: pgrx::Timestamp) -> pgrx::Uuid {
    timestamp_to_uuid(ts, std::u32::MIN.into())
}

#[pg_extern(immutable, parallel_safe)]
fn timestamp_to_uuid_max(ts: pgrx::Timestamp) -> pgrx::Uuid {
    timestamp_to_uuid(ts, std::u32::MAX.into())
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_pgx_uuidv7_now() {
        let g = uuid_generate_v7_now();
        let u: uuid::Uuid = Converter(g).into();
        assert_eq!(7, u.get_version_num());
    }

    #[pg_test]
    fn test_pgx_uuidv7() {
        let pt000: pgrx::Timestamp = pgrx::Timestamp::new(2012, 3, 4, 5, 6, 7.123456789).unwrap();
        let g: pgrx::Uuid = uuid_generate_v7(pt000); // <-- calling
        let u: uuid::Uuid = Converter(g).into();
        assert_eq!(7, u.get_version_num());

        let ut000: uuid::Timestamp = u.get_timestamp().unwrap();
        let (epoch, nanoseconds) = ut000.to_unix();

        let _millis = (epoch * 1000).saturating_add(nanoseconds as u64 / 1_000_000);

        assert_eq!(epoch, 1_330_837_567);
        // Uuid::new_v7 uses milliseconds, not nanoseconds the timestamp structure accepts.
        assert_eq!(nanoseconds, 123_000_000);

        let pt001: pgrx::Timestamp = uuid_to_timestamp(g); // <-- calling
        let pt002: pgrx::Timestamp = pgrx::Timestamp::new(2012, 3, 4, 5, 6, 7.123).unwrap();
        assert_eq!(pt001, pt002);
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
