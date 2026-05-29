#!/bin/bash

set -e

PREFIX="${1:-/usr/local}"

./build.sh --install --prefix "${PREFIX}"