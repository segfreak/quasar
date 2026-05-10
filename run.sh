#!/bin/bash
RUST_LOG=trace cargo run --features hashbrown --bin quassa
dot -Tpng -o preopt-quasar.png preopt-quasar.dot
dot -Tpng -o quasar.png quasar.dot

RUST_LOG=error cargo run --features hashbrown --bin ril
dot -Tpng -o ril.png ril.dot
