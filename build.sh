#!/bin/bash
set -e

ENABLE_LLVM="OFF"
ENABLE_CRANELIFT="ON"
INKWELL_LLVM=""
DO_TEST=0
DO_INSTALL=0
INSTALL_PREFIX="/usr/local/"
FEAR_FEATURES=("fear/binary-ir")
LIBFEAR_FEATURES=("libfear/binary-ir")
CARGO_ARGS=("--release" "--no-default-features")

show_help() {
  echo "Usage: $0 [OPTIONS] [-- CARGO_ARGUMENTS]"
  echo ""
  echo "fear-project build script."
  echo ""
  echo "Options:"
  echo "  -h, --help                    Show this help menu and exit"
  echo "      --enable-llvm=ON/OFF      Enable or disable the LLVM backend       (Default: OFF)"
  echo "      --enable-cranelift=ON/OFF Enable or disable the Cranelift backend  (Default: ON)"
  echo "      --inkwell-llvm=VERSION    Specify the inkwell feature version      (e.g., llvm22-1)"
  echo "      --prefix=PATH             Installation prefix path"
  echo "      --install                 Run installation script after build"
  echo "      --test                    Run tests only"
  echo ""
  echo "Examples:"
  echo "  $0 --enable-llvm=ON --inkwell-llvm=llvm22-1"
  echo ""
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --enable-llvm=*)
      ENABLE_LLVM=$(echo "${1#*=}" | tr '[:lower:]' '[:upper:]')
      shift
      ;;
    --inkwell-llvm=*)
      INKWELL_LLVM=$(echo "${1#*=}" | tr '[:upper:]' '[:lower:]')
      shift
      ;;
    --enable-cranelift=*)
      ENABLE_CRANELIFT=$(echo "${1#*=}" | tr '[:lower:]' '[:upper:]')
      shift
      ;;
    --prefix)
      if [[ -z "$2" || "$2" == -* ]]; then
        echo "error: specify prefix path" >&2
        exit 1
      fi
      INSTALL_PREFIX="$2"
      shift 2
      ;;
    --prefix=*)
      INSTALL_PREFIX="${1#*=}"
      shift
      ;;
    --install)
      DO_INSTALL=1
      shift
      ;;
    --test)
      DO_TEST=1
      shift
      ;;
    -h|--help|-?)
      show_help
      exit 0
      ;;
    --)
      shift
      CARGO_ARGS+=("$@")
      break
      ;;
    *)
      shift
      ;;
  esac
done

echo "LLVM      : $ENABLE_LLVM"
echo "Cranelift : $ENABLE_CRANELIFT"

if [ "$ENABLE_LLVM" = "ON" ]; then
  FEAR_FEATURES+=("fear/llvm")
  LIBFEAR_FEATURES+=("libfear/llvm")
  if [ -n "$INKWELL_LLVM" ]; then
    FEAR_FEATURES+=("inkwell/${INKWELL_LLVM}")
    LIBFEAR_FEATURES+=("inkwell/${INKWELL_LLVM}")
  fi
fi

if [ "$ENABLE_CRANELIFT" = "ON" ]; then
  LIBFEAR_FEATURES+=("libfear/cranelift")
  FEAR_FEATURES+=("fear/cranelift")
fi

FEAR_FEATURES_STR=$(IFS=,; echo "${FEAR_FEATURES[*]}")
LIBFEAR_FEATURES_STR=$(IFS=,; echo "${LIBFEAR_FEATURES[*]}")
ALL_FEATURES_STR="${FEAR_FEATURES_STR},${LIBFEAR_FEATURES_STR}"

if [ "$DO_TEST" = "1" ]; then
  cargo test "${CARGO_ARGS[@]}" --features "$ALL_FEATURES_STR"
else
  cargo build "${CARGO_ARGS[@]}" --features "$ALL_FEATURES_STR"
fi

if [ "$DO_INSTALL" = "1" ]; then
  ./bindings.sh
  INC_DIR="$INSTALL_PREFIX/include"
  LIB_DIR="$INSTALL_PREFIX/lib"
  BIN_DIR="$INSTALL_PREFIX/bin"
  sudo install -d "$INC_DIR"
  sudo install -d "$LIB_DIR"
  sudo install -d "$BIN_DIR"
  sudo cargo install --path "./fear" --root "$INSTALL_PREFIX" --features "$FEAR_FEATURES_STR"
  sudo install -m 644 "./include/fear.h" "$INC_DIR/fear.h"
  sudo install -m 644 "./include/fear.hpp" "$INC_DIR/fear.hpp"
  sudo install -m 755 "./target/release/libfear.so" "$LIB_DIR/"
  sudo install -m 644 "./target/release/libfear.a"  "$LIB_DIR/"
fi
