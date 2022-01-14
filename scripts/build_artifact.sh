#!/bin/bash

set -exuo pipefail

# aarch64,x86_64
export ARCH=$1
export ARTIFACT_ZIP=artifact_$1.zip

export PLATFORM=$ARCH-unknown-linux-musl

function build {
	# release로 뽑지 않으면 바이너리 크기가 크다
	cargo build --target=$PLATFORM --release
}

function archive_aarch64 {
	cp target/$PLATFORM/release/mikoto target/$PLATFORM/release/bootstrap
	zip -r -j $ARTIFACT_ZIP target/$PLATFORM/release/bootstrap
}

function archive_x86_64 {
	zip -r -j $ARTIFACT_ZIP target/$PLATFORM/release/mikoto
}

build
rm -rf $ARTIFACT_ZIP
archive_$ARCH
