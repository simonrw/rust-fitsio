# Restructuring

Date: 2018-04-21

The layout of the crate has now changed. The import paths make a lot
more sense now.

Whereas before, for example `FitsHdu` was stored in `fitsfile.rs`, it
now has it's own location under `hdu.rs`.

This means the typical imports, for all functionality now look like
this:

```rust
use fitsio::FitsFile;
use fitsio::hdu::HduInfo;
use fitsio::images::{ImageDescription, ImageType};
use fitsio::tables::{ColumnDataType, ColumnDescription, FitsRow};
use fitsio::errors::{Result, Error};
```

This may be put into a `prelude` in the future, but it's really not that
much to import, especially if only partial functionality is needed.

## Migrating to v0.14.0

The above types' previous and current locations are listed below for
those transitioning:

* `FitsFile`: no change (`fitsfile`)
* `FitsHdu`: `fitsfile` -> `hdu`
* `HduInfo`: `types` -> `hdu`
* `ImageDescription`: `fitsfile` -> `images`
* `ImageType`: `types` -> `images`
* `ColumnDataType`: `columndescription` -> `tables`
* `ColumnDescription`: `columndescription` -> `tables`
* `FitsRow`: `fitsfile` -> `tables`
* `Result`: no change (`errors`)
* `Error`: no change (`errors`)
