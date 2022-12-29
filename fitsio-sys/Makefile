EXTRA_CLANG_ARGS ?=

all: src/bindings_64.rs src/bindings_32.rs

.PHONY: src/bindings_64.rs
src/bindings_64.rs:
ifeq ($(shell uname -m),x86_64)
	bindgen \
		-o $@ \
		--block-extern-crate \
		--opaque-type fitsfile \
		--opaque-type FITSfile \
		--rust-target "1.47" \
		wrapper.h -- ${EXTRA_CLANG_ARGS}
else
	@echo "bindings_64.rs must be generated on a 64 bit processor"
endif

.PHONY: src/bindings_32.rs
src/bindings_32.rs:
ifeq ($(shell uname -m),armv7l)
	bindgen \
		-o $@ \
		--block-extern-crate \
		--opaque-type fitsfile \
		--opaque-type FITSfile \
		--rust-target "1.47" \
		wrapper.h -- ${EXTRA_CLANG_ARGS}
else ifeq ($(shell uname -m),x86_64)
	bindgen \
		-o $@ \
		--block-extern-crate \
		--opaque-type fitsfile \
		--opaque-type FITSfile \
		--rust-target "1.47" \
		wrapper.h \
		-- -target armv7-unknown-linux-gnueabihf ${EXTRA_CLANG_ARGS}
else
	@echo "bindings_32.rs must be generated on a 32 bit processor"
endif
