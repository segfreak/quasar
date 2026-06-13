-include make.conf

PREFIX ?= /usr/local

LLVM         ?= OFF
CRANELIFT    ?= ON
LLVM_VERSION ?= llvm22-1

RELEASE ?= 0

BUILD_ARGS := --enable-cranelift=$(CRANELIFT)

ifeq ($(LLVM),ON)
  BUILD_ARGS += --with-llvm=$(LLVM_VERSION)
else
  BUILD_ARGS += --enable-llvm=OFF
endif

ifeq ($(RELEASE),1)
  BUILD_ARGS += --release
endif

.PHONY: all build test install clean help

all: build

build:
	./build.sh $(BUILD_ARGS)

test:
	./build.sh --test $(BUILD_ARGS)

install:
	./build.sh $(BUILD_ARGS) --prefix=$(PREFIX) --install

clean:
	@if [ -d "target" ]; then cargo clean; echo "Cargo cache cleaned."; else echo "Nothing to clean."; fi

help:
	@printf "Usage: make [TARGET] [VARIABLE=VALUE]\n"
	@printf "\n"
	@printf "Targets:\n"
	@printf "  all                  Build the project (default)\n"
	@printf "  test                 Run tests\n"
	@printf "  install              Build and install binaries\n"
	@printf "  clean                Remove build artifacts\n"
	@printf "  help                 Display this help message\n"
	@printf "\n"
	@printf "Variables:\n"
	@printf "  RELEASE=1            Build in release mode with optimizations\n"
	@printf "  LLVM=ON|OFF          Enable LLVM backend\n"
	@printf "  LLVM_VERSION=<ver>   Set LLVM version (default: $(LLVM_VERSION))\n"
	@printf "  CRANELIFT=ON|OFF     Enable Cranelift backend\n"
	@printf "  PREFIX=<path>        Installation prefix (default: $(PREFIX))\n"