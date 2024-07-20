# Contributing

Many thanks for your interest in `rust-fitsio`. I appreciate any suggestions or help!

## Local development setup

Either follow the instructions in the [README](./README.md) to support installing locally, otherwise see the [Docker](#docker) instructions.

### Docker

We supply a [Dockerfile](./Dockerfile) which sets up a linux environment that has all packages required for development (i.e. it should be able to run `./bin/test -t all` as the CI tests do). For development on non-Linux platforms this may be more convenient.

```
# change directory into the rust-fitsio root directory
docker build -t <tag> .
docker run --rm -it -v $(pwd):/project <tag> bash
```

## PR checklist

Before submitting a completed PR, make sure the following items have been addressed:

* **ensure all tests pass** - new features require at least one test to demonstrate the behaviour and check for regressions. Changed code must be reflected in the existing tests.
* **update the documentation** - make sure the documentation at the top of `src/lib.rs` is up to date and in sync with the code itself. The previous item should help with the code portions of the documentation.
* **update the changelog** - try to keep with the existing format, and add any additions, changes or removals to the `upstream` section.
* **format the code** - make sure the code has been formatted by `rustfmt` before submitting. I have a [git `pre-push` hook](https://gist.github.com/zofrex/4a5084c49e4aadd0a3fa0edda14b1fa8) which handles this for me.
* **update the features tracking issue** - if relevant, update the [features tracking issue][features-tracking-issue]
* **update the full example** - if new features have been added, or changes made, update the `full_example.rs` example
* **satisfy clippy** - our Github CI will apply `cargo clippy` with warnings treated as errors.

[features-tracking-issue]: https://github.com/simonrw/rust-fitsio/issues/15

---

vim: ft=markdown:textwidth=0:wrap:nocindent
