#!/bin/bash
RUST_LOG=trace cargo run --features hashbrown --bin mirssa
dot -Tpng -o preopt-mirssa.png preopt-mirssa.dot
dot -Tpng -o mirssa.png        mirssa.dot

RUST_LOG=error cargo run --features hashbrown --bin ril
dot -Tpng -o ril.png ril.dot
