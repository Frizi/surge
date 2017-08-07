#!/bin/bash
set -e
cd `dirname $0`/..

cargo build --release --all
./scripts/osx_vst_bundler.sh Pendulum target/release/libpendulum.dylib
./scripts/osx_vst_bundler.sh Fermi target/release/libfermi.dylib

./scripts/test.sh
