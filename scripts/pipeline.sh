#!/bin/bash
set -e
cd `dirname $0`/..

cargo build --release --all
./scripts/osx_vst_bundler.sh Pendulum target/release/libpendulum.dylib
./scripts/test.sh
