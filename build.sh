#!/bin/bash
set -e

ENABLE_LLVM="OFF"
ENABLE_CRANELIFT="ON"
INKWELL_LLVM=""
FEATURES=("libfear/binary-ir" "fear/binary-ir")
CARGO_ARGS=("--no-default-features")

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
  echo ""
  echo "Examples:"
  echo "  $0 --enable-llvm=ON --inkwell-llvm=llvm22-1"
  echo ""
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --enable-llvm=*)
      ENABLE_LLVM="${1#*=}"
      shift
      ;;
    --inkwell-llvm=*)
      INKWELL_LLVM="${1#*=}"
      shift
      ;;
    --enable-cranelift=*)
      ENABLE_CRANELIFT="${1#*=}"
      shift
      ;;
    -h|--help|-?)
      show_help
      exit 0
      ;;
    *)
      CARGO_ARGS+=("$1")
      shift
      ;;
  esac
done

echo "llvm      : $ENABLE_LLVM"
echo "cranelift : $ENABLE_CRANELIFT"

if [ "$ENABLE_LLVM" = "ON" ]; then
  FEATURES+=("libfear/llvm" "fear/llvm")
  if [ -n "$INKWELL_LLVM" ]; then
    FEATURES+=("inkwell/${INKWELL_LLVM}")
  fi
fi

if [ "$ENABLE_CRANELIFT" = "ON" ]; then
  FEATURES+=("libfear/cranelift" "fear/cranelift")
fi

FEATURES_STR=$(IFS=,; echo "${FEATURES[*]}")

cargo build "${CARGO_ARGS[@]}" --features "$FEATURES_STR"

./bindings.sh