#!/bin/bash
set -e
cd `dirname $0`/..

./scripts/mrswatson64 --midi-file arp.mid --output output.wav --plugin Pendulum.vst
