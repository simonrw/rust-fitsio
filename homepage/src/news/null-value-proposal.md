# Null value proposal

Date: 2018-05-05

I want to support `NULL` values in `fitsio`. It is fully supported in
`cfitsio` and is incredibly useful functionality.

It allows the user of the library to _know_ what values are `NULL`
rather than assuming, or having to deal with `NaN` values. Additionally,
integers have no meaningful `NULL` value so capturing the _meaning_
behind a value is not possible.

Naturally, Rust represents the concept of `NULL` with the
[`std::option::Option`][1] type, rather than a custom value that
type-checks as any other type, and can make it's way through code
unhandled. Other people have made better arguments for why an [`Option`]
type is a sensible alternative.

## Solution

One option is for `fitsio` to return a `Vec<Option<T>>` however this
causes non-contiguous memory layout and inflates the size of the vector
([see this discussion][2]).

Another option is to create a new type, which contains the contiguous
`Vec`, and a array storing whether that element is null or not.

A naive implementation looks like:

```rust
struct NullVec<T> {
    contents: Vec<T>,
    is_null: Vec<bool>,
}
```

This can be improved by switching out the `is_null` member for a
[`bitvec`][3]:

```rust
struct NullVec<T> {
    contents: Vec<T>,
    is_null: BitVec,
}
```

This solution gets as as far as storing the actual data, but how to
return it from `read_*` methods? I see two alternatives:

## Integration with fitsio A

This data type must be compatible with all of the read methods. It _is_
possible to write null values to a fits file, but the API is a little
disgusting[^4]. I will shelve _writing_ null values for now.

This restriction means it has to integrate with [`ReadImage`][5] and
[`ReadsCol`][6].

In particular, it needs to integrate with `ReadImage<Vec<T>>` and
`ReadsCol<Vec<T>>`. This means `ReadsCol` has to be re-created to work
on `Vec<T>`.

After this has been done, then I can implement `ReadImage`/`ReadsCol`
for `NullVec<T>` and handle the null values properly.

Hopefully this should not be too much work. The final result I want the
end user to have is:

```rust
let data: NullVec<f64> = hdu.read_col(&mut fptr, "DATA");
let first_value: Option<f64> = data[0];
```

if they care about null values, and

```rust
let data: Vec<f64> = hdu.read_col(&mut fptr, "DATA");
let first_value: f64 = data[0]; // NULL values be damned
```

(i.e. the existing behaviour) if they don't.

I am not sure the [`std::ops::Index`][7] trait supports returning
`Option` types however...

## Integration with fitsio B

The alternative is to support a single return type, but implement a
new trait that returns an `Option` value if the underlying data is not
`NULL`. [`ReadImage`][5] and [`ReadsCol`][6] can then return
implementors of this trait.

This trait will then have to be implemented for all of the available
return types and is compatible with future ones.

I'll have more details in the future.


[1]: https://doc.rust-lang.org/std/option/enum.Option.html
[2]: https://www.reddit.com/r/rust/comments/2x3o3f/what_is_this_performance_implication_of/
[3]: https://crates.io/crates/bit-vec

[^4]: _"substitute the appropriate FITS null value for all elements
  which are equal to the input value of nulval [..]. For integer columns
  the FITS null value is defined by the TNULLn keyword [..]. For
  floating point columns the special IEEE NaN (Not-a-Number) value will
  be written into the FITS file"_

[5]: https://docs.rs/fitsio/0.14.0/fitsio/images/trait.ReadImage.html
[6]: https://docs.rs/fitsio/0.14.0/fitsio/tables/trait.ReadsCol.html
[7]: https://doc.rust-lang.org/std/ops/trait.Index.html
