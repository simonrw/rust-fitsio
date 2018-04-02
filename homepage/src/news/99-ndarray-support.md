# ndarray support

Date: 2018-04-02

The latest feature has been added to the master branch. `fitsio` now
supports reading image data into [`ndarray`] arrays.

Replicating some of the functionality of the Python [numpy] library,
where an array object can be treated like a simple scalar variable.

For example:

```rust
{{#include ../../fitsioexample/src/bin/ndarray_support.rs:9:14}}
```

Currently, only reading images is supported. I do see writing `Array`
objects coming in the future, but it is not currently supported.

I hope to be adding `ndarray` support to a formal versioned release
soon.

## Contributions

- [astrojghu] for seeding the idea in [this thread]

[`ndarray`]: https://crates.io/crates/ndarray
[astrojghu]: https://users.rust-lang.org/u/astrojhgu/summary
[this thread]: https://users.rust-lang.org/t/calling-all-rusty-astronomers/14099
[numpy]: https://pypi.python.org/pypi/numpy
