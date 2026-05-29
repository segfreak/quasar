#!/bin/bash
mkdir -p include/
cbindgen --config cbindgen.toml --crate fearc > include/fear.h