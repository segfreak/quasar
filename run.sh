#!/bin/bash
RUST_LOG=trace cargo run --features hashbrown --bin mirssa
dot -Tpng -o preopt-mirssa.png preopt-mirssa.dot
dot -Tpng -o mirssa.png        mirssa.dot

llc -o mirssa.s mirssa.ll
opt -o mirssa-opt.ll -O3 -S mirssa.ll
llc -o mirssa-opt.s mirssa-opt.ll

# RUST_LOG=debug cargo run --features hashbrown --bin ril
# dot -Tpng -o ril.png ril.dot
