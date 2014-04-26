RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -O
RUST_REPOSITORY ?= ../../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust

lib=build/$(shell rustc --crate-file-name src/lib.rs --crate-type rlib)
src_files=$(wildcard src/*.rs)

all: httpcommon docs

httpcommon: $(lib)

$(lib): $(src_files)
	@mkdir -p build/
	$(RUSTC) $(RUSTFLAGS) src/lib.rs --out-dir=build

docs: doc/httpcommon/index.html

doc/httpcommon/index.html: $(src_files)
	$(RUSTDOC) src/lib.rs

build/test: $(src_files)
	$(RUSTC) $(RUSTFLAGS) --test -o build/test src/lib.rs

build/quicktest: $(src_files)
	$(RUSTC) --test -o build/quicktest src/lib.rs

test: all build/test
	build/test --test
	$(RUSTDOC) -L build --test src/lib.rs

# Can't wait for everything to build, optimised too? OK, you can save some time here.
quicktest: build/quicktest
	build/quicktest --test

clean:
	rm -rf build/ doc/

TAGS: $(src_files)
	ctags -f TAGS --options="$(RUST_CTAGS)" -R src/http

.PHONY: all httpcommon docs clean test quicktest
