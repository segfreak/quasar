#!/bin/bash
cargo run --bin quassa
dot -Tpng -o preopt-quasar.png preopt-quasar.dot
dot -Tpng -o quasar.png quasar.dot