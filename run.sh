#!/bin/bash
cargo run --features hashbrown --bin quassa
dot -Tpng -o preopt-quasar.png preopt-quasar.dot
dot -Tpng -o quasar.png quasar.dot

cargo run --features hashbrown --bin ril
dot -Tpng -o ril.png ril.dot
