
BUILD_IMAGD=ghcr.io/kaznak/pgrx-build:debian_bullseye-pg16-pgrx0.11.2

build:
	docker run --rm -it -v $(PWD):/build -w /build $(BUILD_IMAGD) make build-in-docker

build-in-docker:
	cargo pgrx test
	cargo pgrx package
	cargo build --release
