-include ../make.conf
-include make.conf

CC 		 	 ?= gcc
CFLAGS 	 ?= -std=c99 -D_POSIX_C_SOURCE=200809L
CXX 		 ?= g++
CXXFLAGS ?= -std=c++17
LDFLAGS  ?= -lfear

CSRCS   := $(wildcard *.c)
CXXSRCS := $(wildcard *.cc)
TARGETS := $(CSRCS:.c=.x) $(CXXSRCS:.cc=.x)

all: $(TARGETS)

%.x: %.c
	$(CC) $(CFLAGS) $< $(LDFLAGS) -o $@

%.x: %.cc
	$(CXX) $(CXXFLAGS) $< $(LDFLAGS) -o $@

bazbar.bin: bar.x baz.x
	@./baz.x
	@./bar.x
	fearlink -o bazbar.bin baz.bin bar.bin
	fearopt --opt default bazbar.bin
	feardump bazbar.bin
	fearc --opt default --type object -o bazbar.o bazbar.bin
	cc -O0 -o bazbar.x runners/bazbar.c bazbar.o

clean:
	rm -f $(TARGETS) *.o *.ll *.ssa *.bin *.s

.PHONY: all clean
