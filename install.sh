#!/bin/bash
set -e

PREFIX="${1:-/usr/local}"

INC_DIR="$PREFIX/include"
LIB_DIR="$PREFIX/lib"

install -d "$INC_DIR"
install -d "$LIB_DIR"

install -m 644 "./include/fear.h" "$INC_DIR/fear.h"
install -m 644 "./include/fear.hpp" "$INC_DIR/fear.hpp"

install -m 755 "./target/release/libfearc.so" "$LIB_DIR/"
install -m 644 "./target/release/libfearc.a"  "$LIB_DIR/"
