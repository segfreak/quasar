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

clean:
	rm -f $(TARGETS)

.PHONY: all clean
