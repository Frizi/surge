#!/bin/bash
set -e
cd `dirname $0`

./osx_vst_bundler.sh Pendulum target/release/libpendulum.dylib
./test.sh
