#!/bin/bash
set -e
cd `dirname $0`/..

(cd surgemachinevst && cargo build --release)
./scripts/osx_vst_bundler.sh Pendulum target/release/libpendulum.dylib
./scripts/test.sh
