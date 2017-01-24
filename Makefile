.PHONY: build-in-docker all test integ docker-release

all:
		cargo build

test:
		cargo test

build-in-docker:
		cargo clean
		docker build -t make.local/exec-cmd-builder:latest -f scripts/Dockerfile.build .
		docker run --user=$(shell id -u) -v "$(shell pwd)/target:/rust/app/target" make.local/exec-cmd-builder:latest

docker-release:
		./scripts/build-push-docker-image
