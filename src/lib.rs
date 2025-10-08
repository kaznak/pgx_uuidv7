mod my_converter;

use my_converter::{to_uuid_timestamp_buildpart, Converter};
use pgrx::prelude::*;
use uuid::Uuid;

pgrx::pg_module_magic!();

/// Return the version of given uuid.
#[pg_extern(parallel_safe)]
fn uuid_get_version(uuid: pgrx::Uuid) -> i8 {
    let u: uuid::Uuid = Converter(uuid).into();
    let v = u.get_version_num();
    v as i8
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_get_version"(uuid)
IS 'Return the version of given uuid.';
"#,
    name = "comment_uuid_get_version",
    requires = [uuid_get_version],
);

/// Generate and return a new UUID using the v7 algorithm.
/// The timestamp is the current time.
#[pg_extern(parallel_safe)]
fn uuid_generate_v7_now() -> pgrx::Uuid {
    Converter(Uuid::now_v7()).into()
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_generate_v7_now"()
IS 'Generate and return a new UUID using the v7 algorithm. The timestamp is the current time.';
"#,
    name = "comment_uuid_generate_v7_now",
    requires = [uuid_generate_v7_now],
);

/// Generate and return a new UUID using the v7 algorithm.
/// The timestamp is the given timestamp.
#[pg_extern(parallel_safe)]
fn uuid_generate_v7(ts: pgrx::datum::TimestampWithTimeZone) -> pgrx::Uuid {
    let u = Uuid::new_v7(Converter(ts).into());
    Converter(u).into()
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_generate_v7"(timestamptz)
IS 'Generate and return a new UUID using the v7 algorithm. The timestamp is the given timestamp.';
"#,
    name = "comment_uuid_generate_v7",
    requires = [uuid_generate_v7],
);

/// Generate and return a new UUID using the v7 algorithm.
/// The timestamp is the current time plus the given interval.
#[pg_extern(parallel_safe)]
fn uuid_generate_v7_at_interval(interval: pgrx::datum::Interval) -> pgrx::Uuid {
    // Get current transaction timestamp directly (respects transaction boundaries)
    let current_time = pgrx::datum::datetime_support::now();

    // Add interval directly to timestamp
    let target_time = current_time + interval;

    // Generate UUID v7 with the calculated timestamp
    uuid_generate_v7(target_time)
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_generate_v7_at_interval"(interval)
IS 'Generate and return a new UUID using the v7 algorithm. The timestamp is the current time plus the given interval.';
"#,
    name = "comment_uuid_generate_v7_at_interval",
    requires = [uuid_generate_v7_at_interval],
);

/// Convert a UUID to a timestamptz.
/// The timestamp is the timestamp encoded in the UUID.
/// The timezone is UTC.
#[pg_extern(immutable, parallel_safe)]
fn uuid_to_timestamptz(uuid: pgrx::Uuid) -> Option<pgrx::datum::TimestampWithTimeZone> {
    let u: uuid::Uuid = Converter(uuid).into();
    u.get_timestamp().map(|ts| Converter(ts).into())
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_to_timestamptz"(uuid)
IS 'Convert a UUID to a timestamptz. The timestamp is the timestamp encoded in the UUID. The timezone is UTC.';
"#,
    name = "comment_uuid_to_timestamptz",
    requires = [uuid_to_timestamptz],
);

/// Generate and return a new UUID using the v7 algorithm.
/// The timestamp is the given timestamp.
/// This function is a wrapper around `uuid_generate_v7`.
#[pg_extern(parallel_safe)]
fn timestamptz_to_uuid_v7_random(ts: pgrx::datum::TimestampWithTimeZone) -> pgrx::Uuid {
    uuid_generate_v7(ts)
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "timestamptz_to_uuid_v7_random"(timestamptz)
IS 'Generate and return a new UUID using the v7 algorithm. The timestamp is the given timestamp. This function is a wrapper around `uuid_generate_v7`.';
"#,
    name = "comment_timestamptz_to_uuid_v7_random",
    requires = [timestamptz_to_uuid_v7_random],
);

#[inline]
fn _timestamptz_to_uuid_v7(ts: pgrx::datum::TimestampWithTimeZone, rv: &[u8; 10]) -> pgrx::Uuid {
    let u: uuid::Uuid =
        uuid::Builder::from_unix_timestamp_millis(to_uuid_timestamp_buildpart(ts), rv).into_uuid();
    Converter(u).into()
}

/// Generate and return a new UUID using the v7 algorithm.
/// The timestamp is the given timestamp.
/// The UUID is the minimum UUID that can be generated for the given timestamp.
#[pg_extern(immutable, parallel_safe)]
fn timestamptz_to_uuid_v7_min(ts: pgrx::datum::TimestampWithTimeZone) -> pgrx::Uuid {
    let rv = [0x0 as u8; 10];
    _timestamptz_to_uuid_v7(ts, &rv)
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "timestamptz_to_uuid_v7_min"(timestamptz)
IS 'Generate and return a new UUID using the v7 algorithm. The timestamp is the given timestamp. The UUID is the minimum UUID that can be generated for the given timestamp.';
"#,
    name = "comment_timestamptz_to_uuid_v7_min",
    requires = [timestamptz_to_uuid_v7_min],
);

/// Generate and return a new UUID using the v7 algorithm.
/// The timestamp is the given timestamp.
/// The UUID is the maximum UUID that can be generated for the given timestamp.
#[pg_extern(immutable, parallel_safe)]
fn timestamptz_to_uuid_v7_max(ts: pgrx::datum::TimestampWithTimeZone) -> pgrx::Uuid {
    let rv = [0xff as u8; 10];
    _timestamptz_to_uuid_v7(ts, &rv)
}

extension_sql!(
    r#"
COMMENT ON FUNCTION "timestamptz_to_uuid_v7_max"(timestamptz)
IS 'Generate and return a new UUID using the v7 algorithm. The timestamp is the given timestamp. The UUID is the maximum UUID that can be generated for the given timestamp.';
"#,
    name = "comment_timestamptz_to_uuid_v7_max",
    requires = [timestamptz_to_uuid_v7_max],
);

// PostgreSQL 18 compatibility aliases - only for versions < 18
#[cfg(not(feature = "pg18"))]
/// PostgreSQL 18 compatible alias for uuid_generate_v7_now()
/// Only available when targeting PostgreSQL < 18 to avoid conflicts
#[pg_extern(parallel_safe)]
fn uuidv7() -> pgrx::Uuid {
    uuid_generate_v7_now()
}

#[cfg(not(feature = "pg18"))]
extension_sql!(
    r#"
COMMENT ON FUNCTION "uuidv7"()
IS 'PostgreSQL 18 compatible alias for uuid_generate_v7_now(). Generate and return a new UUID using the v7 algorithm. The timestamp is the current time.';
"#,
    name = "comment_uuidv7",
    requires = [uuidv7],
);

#[cfg(not(feature = "pg18"))]
/// PostgreSQL 18 compatible function with interval parameter
/// Generate UUID v7 with timestamp offset by the given interval from current time
/// Only available when targeting PostgreSQL < 18 to avoid conflicts
#[pg_extern(parallel_safe)]
fn uuidv7_with_interval(interval: pgrx::datum::Interval) -> pgrx::Uuid {
    // Get current transaction timestamp directly (respects transaction boundaries)
    let current_time = pgrx::datum::datetime_support::now();

    // Add interval directly to timestamp
    let target_time = current_time + interval;

    // Generate UUID v7 with the calculated timestamp
    uuid_generate_v7(target_time)
}

#[cfg(not(feature = "pg18"))]
/// PostgreSQL 18 compatible function (overloaded uuidv7 with interval)
/// Only available when targeting PostgreSQL < 18 to avoid conflicts
#[pg_extern(name = "uuidv7", parallel_safe)]
fn uuidv7_interval(interval: pgrx::datum::Interval) -> pgrx::Uuid {
    uuidv7_with_interval(interval)
}

#[cfg(not(feature = "pg18"))]
extension_sql!(
    r#"
COMMENT ON FUNCTION "uuidv7"(interval)
IS 'PostgreSQL 18 compatible function. Generate and return a new UUID using the v7 algorithm. The timestamp is the current time plus the given interval.';
"#,
    name = "comment_uuidv7_interval",
    requires = [uuidv7_interval],
);

#[cfg(not(feature = "pg18"))]
/// PostgreSQL 18 compatible alias for uuid_get_version()
/// Only available when targeting PostgreSQL < 18 to avoid conflicts
#[pg_extern(parallel_safe)]
fn uuid_extract_version(uuid: pgrx::Uuid) -> i16 {
    uuid_get_version(uuid) as i16
}

#[cfg(not(feature = "pg18"))]
extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_extract_version"(uuid)
IS 'PostgreSQL 18 compatible alias for uuid_get_version(). Return the version of given uuid.';
"#,
    name = "comment_uuid_extract_version",
    requires = [uuid_extract_version],
);

#[cfg(not(feature = "pg18"))]
extension_sql!(
    r#"
-- UUIDv1
CREATE DOMAIN uuidv1 AS uuid;

ALTER DOMAIN uuidv1
    ADD CONSTRAINT uuidv1 CHECK (uuid_extract_version(VALUE) = 1);

COMMENT ON DOMAIN uuidv1 IS 'A UUID that is specifically version 1.';

-- UUIDv3
CREATE DOMAIN uuidv3 AS uuid;

ALTER DOMAIN uuidv3
    ADD CONSTRAINT uuidv3 CHECK (uuid_extract_version(VALUE) = 3);

COMMENT ON DOMAIN uuidv3 IS 'A UUID that is specifically version 3.';

-- UUIDv4
CREATE DOMAIN uuidv4 AS uuid;

ALTER DOMAIN uuidv4
    ADD CONSTRAINT uuidv4 CHECK (uuid_extract_version(VALUE) = 4);

COMMENT ON DOMAIN uuidv4 IS 'A UUID that is specifically version 4.';

-- UUIDv5
CREATE DOMAIN uuidv5 AS uuid;

ALTER DOMAIN uuidv5
    ADD CONSTRAINT uuidv5 CHECK (uuid_extract_version(VALUE) = 5);

COMMENT ON DOMAIN uuidv5 IS 'A UUID that is specifically version 5.';

-- UUIDv7
CREATE DOMAIN uuidv7 AS uuid;

ALTER DOMAIN uuidv7 ADD CONSTRAINT uuidv7 CHECK (uuid_extract_version(VALUE) = 7);

COMMENT ON DOMAIN uuidv7 IS 'A UUID that is specifically version 7.';
"#,
    name = "domain_type_uuid_versions",
    requires = [uuid_extract_version],
);

#[cfg(feature = "pg18")]
extension_sql!(
    r#"
-- UUIDv1
CREATE DOMAIN uuidv1 AS uuid;

ALTER DOMAIN uuidv1
    ADD CONSTRAINT uuidv1 CHECK (uuid_get_version(VALUE) = 1);

COMMENT ON DOMAIN uuidv1 IS 'A UUID that is specifically version 1.';

-- UUIDv3
CREATE DOMAIN uuidv3 AS uuid;

ALTER DOMAIN uuidv3
    ADD CONSTRAINT uuidv3 CHECK (uuid_get_version(VALUE) = 3);

COMMENT ON DOMAIN uuidv3 IS 'A UUID that is specifically version 3.';

-- UUIDv4
CREATE DOMAIN uuidv4 AS uuid;

ALTER DOMAIN uuidv4
    ADD CONSTRAINT uuidv4 CHECK (uuid_get_version(VALUE) = 4);

COMMENT ON DOMAIN uuidv4 IS 'A UUID that is specifically version 4.';

-- UUIDv5
CREATE DOMAIN uuidv5 AS uuid;

ALTER DOMAIN uuidv5
    ADD CONSTRAINT uuidv5 CHECK (uuid_get_version(VALUE) = 5);

COMMENT ON DOMAIN uuidv5 IS 'A UUID that is specifically version 5.';

-- UUIDv7
CREATE DOMAIN uuidv7 AS uuid;

ALTER DOMAIN uuidv7 ADD CONSTRAINT uuidv7 CHECK (uuid_get_version(VALUE) = 7);

COMMENT ON DOMAIN uuidv7 IS 'A UUID that is specifically version 7.';
"#,
    name = "domain_type_uuid_versions",
);

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
/// PostgreSQL 18 compatible alias for uuid_to_timestamptz()
/// Only available when targeting PostgreSQL < 17 to avoid conflicts with native uuid_extract_timestamp
#[pg_extern(immutable, parallel_safe)]
fn uuid_extract_timestamp(uuid: pgrx::Uuid) -> Option<pgrx::datum::TimestampWithTimeZone> {
    uuid_to_timestamptz(uuid)
}

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
extension_sql!(
    r#"
COMMENT ON FUNCTION "uuid_extract_timestamp"(uuid)
IS 'PostgreSQL 18 compatible alias for uuid_to_timestamptz(). Convert a UUID to a timestamptz. The timestamp is the timestamp encoded in the UUID. The timezone is UTC. Only available for PostgreSQL < 17.';
"#,
    name = "comment_uuid_extract_timestamp",
    requires = [uuid_extract_timestamp],
);

extension_sql!(
    r#"
CREATE CAST (uuid AS timestamptz) WITH FUNCTION uuid_to_timestamptz(uuid) AS IMPLICIT;
-- timestamptz to uuid is ambiguous, so I don't create it.
"#,
    name = "uuid_casts",
    requires = [uuid_to_timestamptz],
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use pgrx::pg_sys::PgTryBuilder;
    use uuid::{Variant, Version};

    #[pg_test]
    fn test_pgx_uuidv7_now() {
        let g = uuid_generate_v7_now();
        let u: uuid::Uuid = Converter(g).into();
        assert_eq!(7, u.get_version_num());

        let v = uuid_get_version(g);
        assert_eq!(7, v);
    }

    fn gen_pt() -> pgrx::datum::TimestampWithTimeZone {
        pgrx::datum::TimestampWithTimeZone::with_timezone(2012, 3, 4, 5, 6, 7.123456789, "UTC")
            .unwrap()
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

        let pt001: pgrx::datum::TimestampWithTimeZone = uuid_to_timestamptz(g).unwrap(); // <-- calling
        let pt002: pgrx::datum::TimestampWithTimeZone =
            pgrx::datum::TimestampWithTimeZone::with_timezone(2012, 3, 4, 5, 6, 7.123, "UTC")
                .unwrap();
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
        let result = Spi::get_one::<pgrx::Uuid>(
            "SELECT uuid_generate_v7('2012-03-04T05:06:07.123456789+00:00');",
        )
        .unwrap();
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
    fn test_sql() {
        Spi::run(
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
            ",
        )
        .unwrap();

        let ret0 = Spi::connect(|client| {
            client
                .select(
                    "
                SELECT data
                FROM bar
                JOIN foo ON bar.foo_id = foo.id
                ORDER BY data;
                        ",
                    None,
                    &[],
                )
                .unwrap()
                .map(|row| row["data"].value::<String>().unwrap())
                .collect::<Vec<_>>()
        });
        assert!(ret0.len() == 2);
        assert!(ret0[0].is_some());
        assert!(ret0[0].as_ref().unwrap() == "a");
        assert!(ret0[1].is_some());
        assert!(ret0[1].as_ref().unwrap() == "b");

        // join and equal
        let ret1 = Spi::connect(|client| {
            client
                .select(
                    "
                    SELECT data
                    FROM bar
                    JOIN foo ON bar.foo_id = foo.id
                    WHERE foo.id::timestamptz = '2012-03-04T05:06:07.123+00:00';
                    ",
                    None,
                    &[],
                )
                .unwrap()
                .map(|row| row["data"].value::<String>().unwrap())
                .collect::<Vec<_>>()
        });
        assert!(ret1.len() == 1);
        assert!(ret1[0].is_some());
        assert!(ret1[0].as_ref().unwrap() == "a");

        // join and less than
        let ret2 = Spi::connect(|client| {
            client
                .select(
                    "
                    SELECT data
                    FROM bar
                    JOIN foo ON bar.foo_id = foo.id
                    WHERE foo.id::timestamptz < '2012-03-04T05:06:07.123+00:00';
                    ",
                    None,
                    &[],
                )
                .unwrap()
                .map(|row| row["data"].value::<String>().unwrap())
                .collect::<Vec<_>>()
        });
        assert!(ret2.len() == 1);
        assert!(ret2[0].is_some());
        assert!(ret2[0].as_ref().unwrap() == "b");
    }

    #[pg_test]
    fn test_invalid_uuid_version() {
        // Test with UUID v4 (not v7) - should return NULL
        let result = Spi::get_one::<pgrx::datum::TimestampWithTimeZone>(
            "SELECT uuid_to_timestamptz(gen_random_uuid());",
        )
        .unwrap();

        // Should return None for non-v7 UUIDs
        assert!(result.is_none());
    }

    #[pg_test]
    fn test_uuid_generate_v7_with_interval() {
        // Test uuid_generate_v7_at_interval function
        let uuid_past =
            Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7_at_interval(INTERVAL '-1 hour');")
                .unwrap()
                .unwrap();

        let uuid_now = Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7_now();")
            .unwrap()
            .unwrap();

        let uuid_future =
            Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7_at_interval(INTERVAL '1 hour');")
                .unwrap()
                .unwrap();

        // Verify all are version 7
        assert_eq!(uuid_get_version(uuid_past), 7);
        assert_eq!(uuid_get_version(uuid_now), 7);
        assert_eq!(uuid_get_version(uuid_future), 7);

        // Extract timestamps
        let ts_past = uuid_to_timestamptz(uuid_past).unwrap();
        let ts_now = uuid_to_timestamptz(uuid_now).unwrap();
        let ts_future = uuid_to_timestamptz(uuid_future).unwrap();

        // Verify timestamp ordering
        assert!(
            ts_past < ts_now,
            "Past timestamp should be less than current"
        );
        assert!(
            ts_now < ts_future,
            "Current timestamp should be less than future"
        );

        // Verify UUID ordering
        assert!(
            uuid_past < uuid_now,
            "Past UUID should be less than current"
        );
        assert!(
            uuid_now < uuid_future,
            "Current UUID should be less than future"
        );
    }

    #[pg_test]
    fn test_extreme_timestamps() {
        // Test near Unix epoch (1970-01-01)
        let epoch_result =
            Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7('1970-01-01T00:00:00+00:00');")
                .unwrap();
        assert!(epoch_result.is_some());

        // Test year 2038 (32-bit timestamp overflow)
        let y2038_result =
            Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7('2038-01-19T03:14:07+00:00');")
                .unwrap();
        assert!(y2038_result.is_some());

        // Test far future (year 2100)
        let future_result =
            Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7('2100-01-01T00:00:00+00:00');")
                .unwrap();
        assert!(future_result.is_some());

        // Verify timestamp conversion works correctly
        let verify_result = Spi::get_one::<bool>(
            "
            WITH test_uuid AS (
                SELECT uuid_generate_v7('2100-01-01T00:00:00+00:00') AS id
            )
            SELECT 
                uuid_to_timestamptz(id) = '2100-01-01T00:00:00+00:00'::timestamptz AS matches
            FROM test_uuid
            ",
        )
        .unwrap();
        assert!(verify_result.unwrap());
    }

    #[pg_test]
    fn test_null_handling() {
        // Test NULL input for uuid_generate_v7
        let null_timestamp_result =
            Spi::get_one::<pgrx::Uuid>("SELECT uuid_generate_v7(NULL::timestamptz);").unwrap();
        assert!(null_timestamp_result.is_none());

        // Test NULL input for uuid_to_timestamptz
        let null_uuid_result = Spi::get_one::<pgrx::datum::TimestampWithTimeZone>(
            "SELECT uuid_to_timestamptz(NULL::uuid);",
        )
        .unwrap();
        assert!(null_uuid_result.is_none());

        // Test NULL input for timestamptz_to_uuid_v7_min
        let null_min_result =
            Spi::get_one::<pgrx::Uuid>("SELECT timestamptz_to_uuid_v7_min(NULL::timestamptz);")
                .unwrap();
        assert!(null_min_result.is_none());
    }

    #[pg_test]
    fn test_concurrent_uuid_generation() {
        // Generate multiple UUIDs at the same timestamp using SQL
        Spi::run(
            "
            CREATE TEMP TABLE uuid_test AS
            WITH same_time AS (
                SELECT '2023-06-15T12:34:56.789+00:00'::timestamptz AS ts
            )
            SELECT 
                timestamptz_to_uuid_v7_random(ts) AS uuid1,
                timestamptz_to_uuid_v7_random(ts) AS uuid2,
                timestamptz_to_uuid_v7_random(ts) AS uuid3
            FROM same_time;
            ",
        )
        .unwrap();

        // Verify all UUIDs are different
        let unique_count = Spi::get_one::<i64>(
            "SELECT COUNT(DISTINCT uuid_val) FROM (SELECT uuid1 AS uuid_val FROM uuid_test UNION ALL SELECT uuid2 AS uuid_val FROM uuid_test UNION ALL SELECT uuid3 AS uuid_val FROM uuid_test) t;"
        ).unwrap().unwrap();
        assert_eq!(unique_count, 3);

        // Verify they all convert back to the same timestamp
        let same_timestamp_count = Spi::get_one::<i64>(
            "
            SELECT COUNT(DISTINCT ts) FROM (
                SELECT uuid_to_timestamptz(uuid1) AS ts FROM uuid_test
                UNION ALL
                SELECT uuid_to_timestamptz(uuid2) AS ts FROM uuid_test
                UNION ALL
                SELECT uuid_to_timestamptz(uuid3) AS ts FROM uuid_test
            ) t;
            ",
        )
        .unwrap()
        .unwrap();
        assert_eq!(same_timestamp_count, 1);
    }

    #[pg_test]
    fn test_timezone_handling() {
        // Test with different timezone representations
        // Note: uuid_generate_v7 may include random bits, so we test timestamp conversion instead
        let same_timestamp = Spi::get_one::<bool>(
            "
            SELECT 
                uuid_to_timestamptz(uuid_generate_v7('2023-06-15 12:00:00+02:00'::timestamptz)) = 
                '2023-06-15 10:00:00+00:00'::timestamptz AND
                uuid_to_timestamptz(uuid_generate_v7('2023-06-15 10:00:00+00:00'::timestamptz)) = 
                '2023-06-15 10:00:00+00:00'::timestamptz AND
                uuid_to_timestamptz(uuid_generate_v7('2023-06-15 06:00:00-04:00'::timestamptz)) = 
                '2023-06-15 10:00:00+00:00'::timestamptz
            ",
        )
        .unwrap()
        .unwrap();

        // All should convert back to the same UTC timestamp
        assert!(same_timestamp);
    }

    #[cfg(not(feature = "pg18"))]
    #[pg_test]
    fn test_postgresql_18_compatibility() {
        // Test uuidv7() alias
        let uuid_v7 = uuidv7();
        let version = uuid_extract_version(uuid_v7);
        assert_eq!(version, 7i16);

        // Test that aliases produce same results as original functions
        let uuid_orig = uuid_generate_v7_now();
        let _uuid_alias = uuidv7();

        let version_orig = uuid_get_version(uuid_orig);
        let version_alias = uuid_extract_version(uuid_orig);
        assert_eq!(version_orig as i16, version_alias);

        // uuid_extract_timestamp is only available for PG < 17
        #[cfg(not(feature = "pg17"))]
        {
            let ts_orig = uuid_to_timestamptz(uuid_orig);
            let ts_alias = uuid_extract_timestamp(uuid_orig);
            assert_eq!(ts_orig, ts_alias);

            // Test uuid_extract_timestamp() alias
            let timestamp = uuid_extract_timestamp(uuid_v7);
            assert!(timestamp.is_some());
        }
    }

    #[cfg(not(feature = "pg18"))]
    #[pg_test]
    fn test_uuidv7_with_interval() {
        // Test uuidv7 with interval parameter (PostgreSQL 18 compatibility)
        // Test via SQL to match real-world usage patterns
        let result = Spi::get_one::<pgrx::Uuid>("SELECT uuidv7(INTERVAL '-1 hour');").unwrap();
        assert!(result.is_some());

        let uuid_past = result.unwrap();
        let version = uuid_extract_version(uuid_past);
        assert_eq!(version, 7i16);

        // Just verify that timestamp extraction works
        let timestamp = uuid_to_timestamptz(uuid_past);
        assert!(
            timestamp.is_some(),
            "Should be able to extract timestamp from UUIDv7"
        );
    }

    #[cfg(not(feature = "pg18"))]
    #[pg_test]
    fn test_uuidv7_interval_ordering() {
        // Test that UUIDs generated with different intervals maintain proper ordering
        // Test via SQL to match real-world usage patterns
        let uuid_past = Spi::get_one::<pgrx::Uuid>("SELECT uuidv7(INTERVAL '-1 hour')")
            .unwrap()
            .unwrap();
        let uuid_now = Spi::get_one::<pgrx::Uuid>("SELECT uuidv7()")
            .unwrap()
            .unwrap();

        // Verify all are version 7
        assert_eq!(uuid_extract_version(uuid_past), 7i16);
        assert_eq!(uuid_extract_version(uuid_now), 7i16);

        // Verify timestamps can be extracted
        let ts_past = uuid_to_timestamptz(uuid_past);
        let ts_now = uuid_to_timestamptz(uuid_now);
        assert!(ts_past.is_some());
        assert!(ts_now.is_some());

        // Verify timestamp ordering: past < now
        assert!(
            ts_past.unwrap() < ts_now.unwrap(),
            "Past timestamp should be less than current timestamp"
        );

        // Also verify UUID ordering (UUIDv7 should maintain time-based ordering)
        assert!(
            uuid_past < uuid_now,
            "Past UUID should be less than current UUID"
        );
    }

    #[pg_test]
    fn test_uuid_domains_enforce_versions() {
        fn fixture_uuid(seed: u128, version: Version) -> uuid::Uuid {
            let mut builder = uuid::Builder::from_u128(seed);
            builder.set_variant(Variant::RFC4122);
            builder.set_version(version);
            builder.into_uuid()
        }

        fn insert_into_domain(table: &str, uuid: uuid::Uuid) -> bool {
            let insert_sql = format!("INSERT INTO {table} VALUES ('{uuid}');");
            PgTryBuilder::new(|| matches!(Spi::run(&insert_sql), Ok(())))
                .catch_others(|_| false)
                .catch_rust_panic(|_| false)
                .execute()
        }

        let cases = [
            (
                "uuidv1",
                fixture_uuid(0x1111_2222_3333_4444_5555_6666_7777_8888, Version::Mac),
                fixture_uuid(0x9999_AAAA_BBBB_CCCC_DDDD_EEEE_FFFF_0000, Version::Random),
            ),
            (
                "uuidv3",
                fixture_uuid(0x0001_0203_0405_0607_0809_0A0B_0C0D_0E0F, Version::Md5),
                fixture_uuid(0x1112_1314_1516_1718_191A_1B1C_1D1E_1F20, Version::SortRand),
            ),
            (
                "uuidv4",
                fixture_uuid(0xABC0_ABCD_ABCD_ABCD_ABCD_ABCD_ABCD_ABCD, Version::Random),
                fixture_uuid(0x5555_6666_7777_8888_9999_AAAA_BBBB_CCCC, Version::Sha1),
            ),
            (
                "uuidv5",
                fixture_uuid(0x2468_ACE0_1357_9BDF_0246_8ACE_1357_9BDF, Version::Sha1),
                fixture_uuid(0x0F0E_0D0C_0B0A_0908_0706_0504_0302_0100, Version::Random),
            ),
            (
                "uuidv7",
                fixture_uuid(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210, Version::SortRand),
                fixture_uuid(0x1357_9BDF_2468_ACE0_F0E1_D2C3_B4A5_9687, Version::Random),
            ),
        ];

        for (idx, (domain, valid_uuid, invalid_uuid)) in cases.into_iter().enumerate() {
            let table = format!("domain_check_{domain}_{idx}");
            Spi::run(&format!("CREATE TEMP TABLE {table} (id {domain});")).unwrap();

            assert!(
                insert_into_domain(&table, valid_uuid),
                "Expected insert into {table} to succeed for valid UUID {valid_uuid}",
                table = table,
                valid_uuid = valid_uuid
            );

            let stored_version = Spi::get_one::<i16>(&format!(
                "SELECT uuid_extract_version(id) FROM {table} LIMIT 1;"
            ))
            .unwrap()
            .unwrap();
            assert_eq!(
                stored_version,
                valid_uuid.get_version_num() as i16,
                "Stored UUID version should match domain {domain}"
            );

            assert!(
                !insert_into_domain(&table, invalid_uuid),
                "Expected insert into {table} to fail for invalid UUID {invalid_uuid}",
                table = table,
                invalid_uuid = invalid_uuid
            );
        }
    }

    #[cfg(not(any(feature = "pg17", feature = "pg18")))]
    #[pg_test]
    fn test_uuid_extract_timestamp_pg16_only() {
        // Test uuid_extract_timestamp function (only available for PostgreSQL < 17)
        let uuid_v7 = uuid_generate_v7_now();
        let timestamp = uuid_extract_timestamp(uuid_v7);
        assert!(
            timestamp.is_some(),
            "Should be able to extract timestamp from UUIDv7"
        );

        // Test that it returns the same result as uuid_to_timestamptz
        let ts_orig = uuid_to_timestamptz(uuid_v7);
        let ts_alias = uuid_extract_timestamp(uuid_v7);
        assert_eq!(
            ts_orig, ts_alias,
            "uuid_extract_timestamp should match uuid_to_timestamptz"
        );

        // Test with UUID v4 (should return None)
        let uuid_v4 = Spi::get_one::<pgrx::Uuid>("SELECT gen_random_uuid()")
            .unwrap()
            .unwrap();
        let timestamp_v4 = uuid_extract_timestamp(uuid_v4);
        assert!(
            timestamp_v4.is_none(),
            "Should return None for non-timestamp UUIDs"
        );
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
