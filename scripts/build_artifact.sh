#!/bin/bash

set -exuo pipefail

# export PLATFORM=aarch64-unknown-linux-musl
# x86_84로 뽑으면 golang인척 올려도 돌아간다
export PLATFORM=x86_64-unknown-linux-musl

function build {
	# release로 뽑지 않으면 바이너리 크기가 크다
	cargo build --target=$PLATFORM --release
}

function archive {
	rm -rf artifact.zip
	zip -r -j artifact.zip target/$PLATFORM/release/mikoto
}

build
archive
