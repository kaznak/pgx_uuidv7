EXT_NAME=pgx_uuidv7
EXT_VERSION=0.1.0
PKG_NAME=pgx-uuidv7
ARCH=amd64
PG_VERSION=16
PGRX_VERSION=0.11.2
BUILD_IMAGD=ghcr.io/kaznak/pgrx-build:debian_bullseye-pg$(PG_VERSION)-pgrx$(PGRX_VERSION)

build:
	docker run --rm -v $(PWD):/home/rust/build -w /home/rust/build $(BUILD_IMAGD) cargo pgrx package --no-default-features --features pg$(PG_VERSION)
	./scripts/create-package.debian.sh	\
		$(PG_VERSION)	\
		$(EXT_NAME)	\
		$(EXT_VERSION)	\
		$(ARCH)	\
		$(PKG_NAME)

test:
	docker run --rm -v $(PWD):/home/rust/build -w /home/rust/build $(BUILD_IMAGD) cargo pgrx test

clean:
	sudo rm -rf ./target/release/pgx_uuidv7-pg$(PG_VERSION)
