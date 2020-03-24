all: src/bindings.rs

src/bindings.rs: wrapper.h Makefile
	bindgen \
		-o $@ \
		--block-extern-crate \
		--opaque-type fitsfile \
		--opaque-type FITSfile \
		--rust-target "1.0" \
		$<
