# pgx_uuidv7

An extension for PostgreSQL that implements UUIDv7 with basic features.

## Features

- Generate or Cast to UUIDv7
- Cast from UUIDv7 to timestamptz
- PostgreSQL 18 compatibility (provides compatible function names)

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

Generate with interval offset from current time:

```sql
SELECT uuid_generate_v7_at_interval(INTERVAL '-1 hour');  -- 1 hour ago
SELECT uuid_generate_v7_at_interval(INTERVAL '30 minutes');  -- 30 minutes from now
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

## PostgreSQL 18 Compatibility

This extension provides PostgreSQL 18 compatible function names as aliases:

```sql
-- PostgreSQL 18 compatible functions (available only when targeting PostgreSQL < 18)
SELECT uuidv7();                           -- alias for uuid_generate_v7_now()
SELECT uuidv7(INTERVAL '-1 hour');         -- generate UUID with timestamp offset
SELECT uuid_extract_version(some_uuid);   -- alias for uuid_get_version()
SELECT uuid_extract_timestamp(some_uuid); -- alias for uuid_to_timestamptz()
```

**Note**: These PostgreSQL 18 compatible functions are automatically excluded when building for PostgreSQL 18 to avoid conflicts with native functions. They are only available when targeting PostgreSQL versions prior to 18.

This allows for easy migration to PostgreSQL 18 when it becomes available.

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

**Note**: When building for PostgreSQL 18 (`--features pg18`), the PostgreSQL 18 compatible functions (`uuidv7()`, `uuid_extract_version()`, `uuid_extract_timestamp()`) will be automatically excluded to prevent conflicts with PostgreSQL 18's native UUIDv7 functions.

## Release Workflow

This project uses GitHub Actions for automated releases:

### Creating a Release

#### 1. Prepare Release

Before creating a release, update the version:

**Update version in `Cargo.toml`:**
```toml
[package]
version = "0.1.4"  # Update to new version
```

**Commit the version update:**
```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.4"
git push
```

#### 2. Create and Push Tag

```bash
git tag v0.1.4
git push origin v0.1.4
```

#### 3. Monitor Automatic Build

The workflow will automatically:
- Build packages for PostgreSQL 16 and 17
- Create Debian packages (`.deb` files)
- Generate a draft release on GitHub with auto-generated release notes

**Wait for workflow completion:**
- Go to the [Actions tab](https://github.com/kaznak/pgx_uuidv7/actions) on GitHub
- Monitor the "Release" workflow triggered by your tag
- Ensure all jobs complete successfully (✅)
- If any job fails (❌), investigate and fix the issues before proceeding

#### 4. Review and Verify Release

Before publishing, verify the build results:

**Check the draft release:**
- Go to GitHub Releases page
- Confirm the draft release was created with your tag version
- Verify both packages are attached:
  - `pgx-uuidv7-16-amd64-linux-gnu.deb`
  - `pgx-uuidv7-17-amd64-linux-gnu.deb`

**Optional: Test the packages locally:**
```bash
# Download and test one of the packages
wget https://github.com/kaznak/pgx_uuidv7/releases/download/v0.1.4/pgx-uuidv7-16-amd64-linux-gnu.deb
# Test installation in a clean environment
```

#### 5. Complete Release

- **Write release notes directly on GitHub** describing:
  - New features
  - Bug fixes
  - Breaking changes (if any)
  - Installation/upgrade instructions
  - Known issues (if any)
- **Review all information one final time**
- Click "Publish release" to make it public

### Supported PostgreSQL Versions

The release workflow builds packages for:
- PostgreSQL 16 (`pgx-uuidv7-16-amd64-linux-gnu.deb`)
- PostgreSQL 17 (`pgx-uuidv7-17-amd64-linux-gnu.deb`)

### Release Notes

Release notes are written directly on GitHub during the release process. The workflow automatically generates initial notes from recent commits and PRs, which can then be edited to provide clear, user-friendly descriptions of changes.

###  OSSP UUID library is not well maintained.

https://www.postgresql.org/docs/16/uuid-ossp.html#UUID-OSSP-BUILDING
