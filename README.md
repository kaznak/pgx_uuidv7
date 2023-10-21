# pgx_uuidv7

An extension for PostgreSQL that implements UUIDv7 with basic features.

## Features

- Generate or Cast to UUIDv7
- Cast from UUIDv7 to timestamptz

## Examples

### Simple Generation

Generate for present time:

```sql
SELECT uuid_generate_v7_now();
```

Generate for specific time:

```sql
SELECT uuid_generate_v7('2012-03-04T05:06:07.123456789+00:00');
```

### Cast to compare with timestamptz

Preparation:

```sql
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
```

Check equality:

```sql
SELECT data
FROM bar
JOIN foo ON bar.foo_id = foo.id
WHERE foo.id::timestamptz = '2012-03-04T05:06:07.123+00:00';
```

Narrow down by range:

```sql
SELECT data
FROM bar
JOIN foo ON bar.foo_id = foo.id
WHERE foo.id::timestamptz < '2012-03-04T05:06:07.123+00:00';
```

## References

uses:

- [pgrx v0.11.0](https://github.com/pgcentralfoundation/pgrx)([docs](https://docs.rs/pgrx/0.11.0/pgrx/index.html))
    - install this into your environment to develop this extension.
- uuid([docs](https://docs.rs/uuid/1.4.1/uuid/index.html))

lots of code is copied and modified from these following repositories:

- [pg_uuidv7](https://github.com/craigpastro/pg_uuidv7)
- [pgx_ulid](https://github.com/pksunkara/pgx_ulid)

Thank you.

## Memo

### Build commands

```bash
PG_VERSION=16 # set postgresql major version
cargo pgrx package --no-default-features --features pg$PG_VERSION --pg-config $(ls ~/.pgrx/$PG_VERSION.*/pgrx-install/bin/pg_config 2>/dev/null | tail -n1)
```
