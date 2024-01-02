EXT_NAME=pgx_uuidv7 # must be read from Cargo.toml
EXT_VERSION=0.1.1 # must be read from Cargo.toml
PKG_NAME=pgx-uuidv7 # must be converted from EXT_NAME
ARCH=amd64
PG_VERSION=16
PGRX_VERSION=0.11.2 # must be read from Cargo.toml
BUILD_IMAGD=ghcr.io/kaznak/pgrx-build:debian_bullseye-pg$(PG_VERSION)-pgrx$(PGRX_VERSION)

build:
	id
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGD) \
	./scripts/create-package.debian.sh	\
		$(PG_VERSION)	\
		$(EXT_NAME)	\
		$(EXT_VERSION)	\
		$(ARCH)	\
		$(PKG_NAME)

test:
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGD) cargo pgrx test

clean:
	cargo clean
	sudo rm -rf ./target/release/*.debian_package_tmp
