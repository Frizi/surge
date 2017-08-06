#!/bin/bash
set -e
cd `dirname $0`/..

mkdir -p tmp

./scripts/mrswatson64 --midi-file testdata/arp.mid --output tmp/arp.wav --plugin Pendulum.vst,testdata/unison1.fxp
./scripts/mrswatson64 --midi-file testdata/lownote.mid --output tmp/lownote.wav --plugin Pendulum.vst,testdata/pure_saw.fxp
