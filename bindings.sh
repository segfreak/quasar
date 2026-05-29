#!/bin/bash
mkdir -p include/
cbindgen --config cbindgen.toml --crate libfear > include/fear.h