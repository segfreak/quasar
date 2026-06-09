
CC 		 	= cc
CFLAGS 	= -std=c99 -D_POSIX_C_SOURCE=200809L
LDFLAGS = -lfear

SRCS    = $(wildcard *.c)
TARGETS = $(SRCS:.c=.x)

all: $(TARGETS)

%.x: %.c
	$(CC) $(CFLAGS) $< $(LDFLAGS) -o $@

clean:
	rm -f $(TARGETS)

.PHONY: all clean
