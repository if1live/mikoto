#!/bin/bash
# 프로젝트 최상위에서 실행

set -exuo pipefail

OUT=artifacts/ncc

pnpm exec ncc build ./src/index.ts \
	--external better-sqlite3 \
	--external aws-sdk \
	--no-source-map-register \
	--no-cache \
	--source-map \
	--transpile-only \
	--out $OUT
