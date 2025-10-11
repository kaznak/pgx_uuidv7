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

extension_sql!(
    r#"
-- uuid → bytea
CREATE FUNCTION uuid_to_bytea(u uuid)
RETURNS bytea LANGUAGE internal IMMUTABLE STRICT AS 'uuid_send';

COMMENT ON FUNCTION "uuid_to_bytea"(uuid)
IS 'Convert a uuid to bytea.';

-- bytea → uuid
CREATE FUNCTION bytea_to_uuid(b bytea)
RETURNS uuid LANGUAGE internal IMMUTABLE STRICT AS 'uuid_recv';

COMMENT ON FUNCTION "bytea_to_uuid"(bytea)
IS 'Convert a bytea to uuid.';

-- キャスト経路を登録
CREATE CAST (uuid AS bytea)
  WITH FUNCTION uuid_to_bytea(uuid)
  AS ASSIGNMENT;

CREATE CAST (bytea AS uuid)
  WITH FUNCTION bytea_to_uuid(bytea)
  AS ASSIGNMENT;
"#,
    name = "uuid_bytea_converters"
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
    // requires = [uuid_extract_version],
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
mod tests;

#[cfg(test)]
pub use tests::pg_test;
