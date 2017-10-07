# Contributing

Many thanks for your interest in `rust-fitsio`. I appreciate any suggestions or help!

## PR checklist

Before submitting a completed PR, make sure the following items have been addressed:

* **ensure all tests pass** - new features require at least one test to demonstrate the behaviour and check for regressions. Changed code must be reflected in the existing tests.
* **update the documentation** - make sure the documentation at the top of `src/lib.rs` is up to date and in sync with the code itself. The previous item should help with the code portions of the documentation.
* **update the changelog** - try to keep with the existing format, and add any additions, changes or removals to the `upstream` section.
* **format the code** - make sure the code has been formatted by `rustfmt` before submitting. I have a [git `pre-push` hook](https://gist.github.com/zofrex/4a5084c49e4aadd0a3fa0edda14b1fa8) which handles this for me.