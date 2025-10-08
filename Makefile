PG_VERSION=17
ARCH=amd64

PGRX_VERSION=$(shell yq -r -o json '.dependencies.pgrx' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+')
# debian のバージョンはメインの使用環境である cloudnativepg のイメージの debian のバージョンに合わせる
BUILD_IMAGE=ghcr.io/kaznak/pgrx-build:debian_bookworm-pg$(PG_VERSION)-pgrx$(PGRX_VERSION)

build:
	id
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGE) \
	./scripts/create-package.debian.sh	\
		$(PG_VERSION)	\
		$(ARCH)

test:
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGE)	\
		cargo pgrx test  --no-default-features --features pg$(PG_VERSION)

clean:
	cargo clean
	docker image rm $(BUILD_IMAGE) || true
	sudo rm -rf ./target/release/*.debian_package_tmp
