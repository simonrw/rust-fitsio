build:
	cargo check --manifest-path fitsioexample/Cargo.toml
	mdbook build

publish: build
	ghp ./book
	git push -fq origin gh-pages --no-verify
