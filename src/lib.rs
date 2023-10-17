mod my_converter;

use my_converter::{to_uuid_timestamp_buildpart, Converter};
use pgrx::prelude::*;
use uuid::Uuid;

pgrx::pg_module_magic!();

#[pg_extern(parallel_safe)]
fn uuid_generate_v7_now() -> pgrx::Uuid {
    Converter(Uuid::now_v7()).into()
}

#[pg_extern(parallel_safe)]
fn uuid_generate_v7(ts: pgrx::TimestampWithTimeZone) -> pgrx::Uuid {
    let u = Uuid::new_v7(Converter(ts).into());
    Converter(u).into()
}

#[pg_extern(immutable, parallel_safe)]
fn uuid_to_timestamptz(uuid: pgrx::Uuid) -> Option<pgrx::TimestampWithTimeZone> {
    let u: uuid::Uuid = Converter(uuid).into();
    match u.get_timestamp() {
        Some(ts) => Some(Converter(ts).into()),
        None => None,
    }
}

#[pg_extern(parallel_safe)]
fn timestamptz_to_uuid_v7_random(ts: pgrx::TimestampWithTimeZone) -> pgrx::Uuid {
    uuid_generate_v7(ts)
}

// #[pg_extern(immutable, parallel_safe)] // TODO make this public
fn timestamptz_to_uuid_v7(ts: pgrx::TimestampWithTimeZone, rv: u128) -> pgrx::Uuid {
    let u: uuid::Uuid = uuid::Builder::from_unix_timestamp_millis(
        to_uuid_timestamp_buildpart(ts),
        rv.to_be_bytes()[..10].try_into().unwrap(),
    )
    .into_uuid();
    Converter(u).into()
}

#[pg_extern(immutable, parallel_safe)]
fn timestamptz_to_uuid_v7_min(ts: pgrx::TimestampWithTimeZone) -> pgrx::Uuid {
    timestamptz_to_uuid_v7(ts, std::u128::MIN)
}

#[pg_extern(immutable, parallel_safe)]
fn timestamptz_to_uuid_v7_max(ts: pgrx::TimestampWithTimeZone) -> pgrx::Uuid {
    timestamptz_to_uuid_v7(ts, std::u128::MAX)
}

extension_sql!(
    r#"
CREATE CAST (uuid AS timestamptz) WITH FUNCTION uuid_to_timestamptz(uuid) AS IMPLICIT;
-- timestamptz to uuid is ambiguous, so I don't create it.
"#,
    name = "uuid_casts"
);

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

    fn gen_pt() -> pgrx::TimestampWithTimeZone {
        pgrx::TimestampWithTimeZone::with_timezone(2012, 3, 4, 5, 6, 7.123456789, "UTC").unwrap()
    }

    #[pg_test]
    fn test_pgx_uuidv7_new() {
        let pt = gen_pt();
        let g: pgrx::Uuid = uuid_generate_v7(pt); // <-- calling
        let u: uuid::Uuid = Converter(g).into();
        assert_eq!(7, u.get_version_num());

        let ut000: uuid::Timestamp = u.get_timestamp().unwrap();
        let (epoch, nanoseconds) = ut000.to_unix();

        assert_eq!(epoch, 1_330_837_567);
        // Uuid::new_v7 uses milliseconds, not nanoseconds the timestamp structure accepts.
        assert_eq!(nanoseconds, 123_000_000);

        let pt001: pgrx::TimestampWithTimeZone = uuid_to_timestamptz(g).unwrap(); // <-- calling
        let pt002: pgrx::TimestampWithTimeZone =
            pgrx::TimestampWithTimeZone::with_timezone(2012, 3, 4, 5, 6, 7.123, "UTC").unwrap();
        assert_eq!(pt001, pt002);
    }

    #[pg_test]
    fn test_pgx_uuidv7_min() {
        let pt = gen_pt();
        let u_min: pgrx::Uuid = timestamptz_to_uuid_v7_min(pt); // <-- calling
        let u: uuid::Uuid = Converter(u_min).into();
        assert_eq!(7, u.get_version_num());
    }

    #[pg_test]
    fn test_pgx_uuidv7_max() {
        let pt = gen_pt();
        let u_min: pgrx::Uuid = timestamptz_to_uuid_v7_max(pt); // <-- calling
        let u: uuid::Uuid = Converter(u_min).into();
        assert_eq!(7, u.get_version_num());
    }

    #[pg_test]
    fn test_pgx_uuidv7_order() {
        let pt = gen_pt();
        let u_min: pgrx::Uuid = timestamptz_to_uuid_v7_min(pt); // <-- calling
        let u_rnd: pgrx::Uuid = timestamptz_to_uuid_v7_random(pt); // <-- calling
        let u_max: pgrx::Uuid = timestamptz_to_uuid_v7_max(pt); // <-- calling
        assert!(u_min < u_max);
        assert!(u_min <= u_rnd);
        assert!(u_rnd <= u_max);
    }

    #[pg_test]
    fn test_generate_now() {
        let result = Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7_now();").unwrap();
        assert!(result.is_some());
        let u: uuid::Uuid = Converter(result.unwrap()).into();
        assert_eq!(7, u.get_version_num());
    }

    #[pg_test]
    fn test_generate_new() {
        let result = Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7('2012-03-04T05:06:07.123456789+00:00');").unwrap();
        assert!(result.is_some());
        let pu = result.unwrap();
        let u: uuid::Uuid = Converter(pu).into();
        assert_eq!(7, u.get_version_num());
        let ut000: uuid::Timestamp = u.get_timestamp().unwrap();
        let (epoch, nanoseconds) = ut000.to_unix();
        assert_eq!(epoch, 1_330_837_567);
        // Uuid::new_v7 uses milliseconds, not nanoseconds the timestamp structure accepts.
        assert_eq!(nanoseconds, 123_000_000);
    }

    #[pg_test]
    fn test_sql0() {
        let result = Spi::get_one::<String>(
            "
            CREATE TABLE foo (
                id uuid,
                data TEXT
            );

            CREATE TABLE bar (
                id uuid default uuid_generate_v7_now(),
                foo_id uuid
            );

            INSERT INTO foo
            values (
                uuid_generate_v7('2012-03-04T05:06:07.123456789+00:00'),
                'a'
            ), (
                uuid_generate_v7('2001-12-03T04:05:06.123456789+00:00'),
                'b'
            );

            INSERT INTO bar (foo_id) SELECT id FROM foo;

            SELECT data
            FROM bar
            JOIN foo ON bar.foo_id = foo.id
            WHERE foo.id::timestamptz = '2012-03-04T05:06:07.123+00:00';
            ",
        ).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap() == "a");
    }

    #[pg_test]
    fn test_sql1() {
        let result = Spi::get_one::<String>(
            "
            CREATE TABLE foo (
                id uuid,
                data TEXT
            );

            CREATE TABLE bar (
                id uuid default uuid_generate_v7_now(),
                foo_id uuid
            );

            INSERT INTO foo
            values (
                uuid_generate_v7('2012-03-04T05:06:07.123456789+00:00'),
                'a'
            ), (
                uuid_generate_v7('2001-12-03T04:05:06.123456789+00:00'),
                'b'
            );

            INSERT INTO bar (foo_id) SELECT id FROM foo;

            SELECT data
            FROM bar
            JOIN foo ON bar.foo_id = foo.id
            WHERE foo.id::timestamptz < '2012-03-04T05:06:07.123+00:00';
            ",
        ).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap() == "b");
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
