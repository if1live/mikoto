#!/bin/bash

set -exuo pipefail

# aarch64,x86_64
export PLATFORM=aarch64-unknown-linux-musl

function build {
	TARGET=$1
	# release로 뽑지 않으면 바이너리 크기가 크다
	cargo build --bin $1 --target=$PLATFORM --release
}

function archive {
	TARGET=$1
	mkdir -p artifacts/$TARGET && rm -rf artifacts/$TARGET/*
	cp target/$PLATFORM/release/$TARGET artifacts/$TARGET/bootstrap
	rm -rf artifacts/artifact_$TARGET.zip
	zip -r -j artifacts/artifact_$TARGET.zip artifacts/$TARGET/bootstrap
}

build main
build sub

archive main
archive sub
