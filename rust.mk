RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -O
RUST_REPOSITORY ?= ../../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust

all: $(CRATES) docs

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))

# For each crate, we get these variables:
#
# SRC_cratename
#     list of all .rs files in src/cratename (automatically determined)
#
# DEP_cratename
#     list of crate names that it depends on
#
# DEP_LIB_cratename
#     list of crate .rlib files that it depends on (derived from DEP_cratename)
#
# LIB_cratename
#     filename of .rlib file that will be built; a valid rule (automatically determined)
#
# And these friendly rules:
#
# cratename
#     Build the .so/whatever and .rlib
#
# cratename-docs
#     Build the crate's documentation
#
# cratename-test
#     Build and run the crate's tests, including doc tests
#
# cratename-doctest
#     Quickly run the crate's doc tests
#
# cratename-quicktest
#     Quickly run the crate's tests (unoptimised and doesn't depend on the crate being built

define CRATE_RULES
SRC_$(1) := $$(call rwildcard,src/$(1)/,*.rs)
LIB_$(1) := build/$$(shell rustc --crate-file-name src/$(1)/lib.rs --crate-type rlib)
ifeq ($$(LIB_$(1)),build/)
# We may not have rustc or the lib.rs file may be broken.
# But don't break the rules on that account.
LIB_$(1) := build/lib$(1).dummy
endif

$(1): $$(LIB_$(1))

$$(LIB_$(1)): $$(SRC_$(1)) $$(DEP_LIB_$(1))
	@mkdir -p build/
	$$(RUSTC) $$(RUSTFLAGS) src/$(1)/lib.rs --out-dir=build -L build

$(1)-docs: doc/$(1)/index.html

doc/$(1)/index.html: $$(SRC_$(1)) $$(DEP_LIB_$(1))
	$$(RUSTDOC) src/$(1)/lib.rs -L build

build/$(1)-test: $$(SRC_$(1)) $$(DEP_LIB_$(1))
	$$(RUSTC) $$(RUSTFLAGS) --test -o build/$(1)-test src/$(1)/lib.rs -L build

build/$(1)-quicktest: $$(SRC_$(1)) $$(DEP_LIB_$(1))
	$$(RUSTC) --test -o build/$(1)-quicktest src/$(1)/lib.rs -L build

$(1)-test: $(1) $(1)-doctest build/$(1)-test
	build/$(1)-test --test

$(1)-doctest: $$(SRC_$(1)) $$(DEP_LIB_$(1))
	$$(RUSTDOC) -L build --test src/$(1)/lib.rs

# Can't wait for everything to build, optimised too? OK, you can save some time here.
$(1)-quicktest: build/$(1)-quicktest
	build/$(1)-quicktest --test

.PHONY: $(1) $(1)-test $(1)-doctest $(1)-quicktest $(1)-docs

DEP_LIB_$(1) := $$(foreach dep,$$(DEP_$(1)), $$(LIB_$$(dep)))
SRC_ALL := $$(SRC_ALL) $$(SRC_$(1))

endef

$(foreach crate,$(CRATES),$(eval $(call CRATE_RULES,$(crate))))

docs: $(foreach crate,$(CRATES), doc/$(crate)/index.html)

test: $(CRATES) $(foreach crate,$(CRATES), $(crate)-test)

quicktest: $(foreach crate,$(CRATES), $(crate)-quicktest)

clean:
	rm -rf build/ doc/

TAGS: $(SRC_ALL)
	ctags -f TAGS --options="$(RUST_CTAGS)" --language=rust -R src/

.PHONY: all docs clean test quicktest
