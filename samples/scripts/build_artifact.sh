#!/bin/bash
# 프로젝트 최상위에서 실행

set -exuo pipefail

function clean {
	mkdir -p artifacts/ && rm -rf artifacts/*
}

function build {
	./scripts/build_ncc.sh
}

function archive {
	pushd artifacts
	zip -r artifact.zip src/stub* src/*.js src/*.json
	popd
}

clean
build
archive
