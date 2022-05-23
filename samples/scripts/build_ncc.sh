#!/bin/bash

set -exuo pipefail

OUT=artifacts/src

pnpm exec ncc build ./src/index.js \
	--external aws-sdk \
	--no-source-map-register \
	--no-cache \
	--source-map \
	--transpile-only \
	--out $OUT
