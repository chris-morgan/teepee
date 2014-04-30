RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -O
RUST_REPOSITORY ?= ../../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust

CRATES=httpcommon

all: $(CRATES) docs

# Recursive wildcard function
# http://blog.jgc.org/2011/07/gnu-make-recursive-wildcard-function.html
rwildcard=$(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) \
  $(filter $(subst *,%,$2),$d))

define CRATE_RULES
SRC_$(1) := $$(call rwildcard,src/$(1)/,*.rs)
LIB_$(1) := build/$$(shell rustc --crate-file-name src/$(1)/lib.rs --crate-type rlib)
ifeq ($$(LIB_$(1)),build/)
# We may not have rustc or the lib.rs file may be broken.
# But don't break the rules on that account.
LIB_$(1) := build/lib$(1).dummy
endif

$(1): $$(LIB_$(1))

$$(LIB_$(1)): $$(SRC_$(1)) $$(DEP_$(1))
	@mkdir -p build/
	$$(RUSTC) $$(RUSTFLAGS) src/$(1)/lib.rs --out-dir=build -L build

$(1)-docs: doc/$(1)/index.html

doc/$(1)/index.html: $$(SRC_$(1)) $$(DEP_$(1))
	$$(RUSTDOC) src/$(1)/lib.rs -L build

build/$(1)-test: $$(SRC_$(1)) $$(DEP_$(1))
	$$(RUSTC) $$(RUSTFLAGS) --test -o build/$(1)-test src/$(1)/lib.rs -L build

build/$(1)-quicktest: $$(SRC_$(1)) $$(DEP_$(1))
	$$(RUSTC) --test -o build/$(1)-quicktest src/$(1)/lib.rs -L build

$(1)-test: $(1) build/$(1)-test
	build/$(1)-test --test
	$$(RUSTDOC) -L build --test src/$(1)/lib.rs

# Can't wait for everything to build, optimised too? OK, you can save some time here.
$(1)-quicktest: build/$(1)-quicktest
	build/$(1)-quicktest --test

.PHONY: $(1) $(1)-test $(1)-quicktest $(1)-docs

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
