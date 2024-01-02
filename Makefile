PG_VERSION=16
ARCH=amd64

PGRX_VERSION=$(shell yq -r -o json '.dependencies.pgrx' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+')
BUILD_IMAGD=ghcr.io/kaznak/pgrx-build:debian_bullseye-pg$(PG_VERSION)-pgrx$(PGRX_VERSION)

build:
	id
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGD) \
	./scripts/create-package.debian.sh	\
		$(PG_VERSION)	\
		$(ARCH)

test:
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGD) cargo pgrx test

clean:
	cargo clean
	sudo rm -rf ./target/release/*.debian_package_tmp
