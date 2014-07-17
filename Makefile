RUSTC_FLAGS ?= -W missing-doc -W non-uppercase-statics
PREFIX ?= /usr/local
TARGET ?= target

ifeq ($(wildcard rustc/bin),)
export RUSTC := rustc
else
export RUSTC := $(CURDIR)/rustc/bin/rustc
export LD_LIBRARY_PATH := $(CURDIR)/rustc/lib:$(LD_LIBRARY_PATH)
export DYLD_LIBRARY_PATH := $(CURDIR)/rustc/lib:$(DYLD_LIBRARY_PATH)
endif

export PATH := $(CURDIR)/rustc/bin:$(PATH)

SRC = $(shell find src -name '*.rs')

ifeq ($(OS),Windows_NT)
X = .exe
endif

LIB = $(TARGET)/$(shell $(RUSTC) --print-file-name src/lib.rs)
TESTDIR = $(TARGET)/test

all: lib

$(TARGET)/:
	mkdir -p $@

$(TESTDIR)/:
	mkdir -p $@

$(LIB): $(SRC) | $(TARGET)/
	$(RUSTC) $(RUSTC_FLAGS) --out-dir $(TARGET) src/lib.rs

lib: $(LIB)

$(TESTDIR)/whitebase: $(SRC) | $(TESTDIR)/
	$(RUSTC) --test -g $(RUSTC_FLAGS) -o $@ src/lib.rs

test: test-unit

clean:
	rm -rf $(TARGET)

install: $(LIB)
	install -d $(PREFIX)/lib
	install $(LIB) $(PREFIX)/lib

# Setup phony tasks
.PHONY: all clean test test-unit lib

# Disable unnecessary built-in rules
.SUFFIXES:
