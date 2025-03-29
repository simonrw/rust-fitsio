CFITSIO_VERSION := 4.5.0

.PHONY: fetch-cfitsio-source
fetch-cfitsio-source:
	cargo run -p fitsio-src-fetcher -- $(CFITSIO_VERSION) --output fitsio-sys/ext/cfitsio
