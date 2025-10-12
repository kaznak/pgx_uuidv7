PG_VERSION=17
DEBIAN_CODENAME=bullseye
# debian のバージョンはメインの使用環境である cloudnativepg のイメージの debian のバージョンに合わせる
ARCH=amd64

# PGRX_VERSION=0.16.1 # new ターゲットを作る時には即値で指定する
PGRX_VERSION=$(shell yq -r -o json '.dependencies.pgrx' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+')
BUILD_IMAGE=ghcr.io/kaznak/pgrx-build:debian_$(DEBIAN_CODENAME)-pg$(PG_VERSION)-pgrx$(PGRX_VERSION)

# new:
# 	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGE) \
# 	cargo pgrx new pgx_uuidv7

build:
	id
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGE) \
	./scripts/create-package.debian.sh	\
		$(PG_VERSION)	\
		$(ARCH)	\
		$(DEBIAN_CODENAME)

test:
	docker run --rm -v $(PWD):/checkout -w /checkout $(BUILD_IMAGE)	\
		cargo pgrx test  --no-default-features --features pg$(PG_VERSION)

clean:
	cargo clean
	docker image rm $(BUILD_IMAGE) || true
	sudo rm -rf ./target/release/*.debian_package_tmp
