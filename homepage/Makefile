build:
	cargo run --manifest-path fitsioexample/Cargo.toml --bin ndarray_support
	mdbook build

publish: build
	ghp ./book
	git push -fq origin gh-pages --no-verify

server:
	mdbook serve
