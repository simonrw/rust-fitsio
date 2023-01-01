# cfitsio source code

The directory here contains the C source code for cfitsio, untarred from [its
host website](https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/). It is
provided here to allow the rust-fitsio crate to statically compile the C
library, thereby alleviating the need for cfitsio to be present outside of the
Rust ecosystem. However, a C compiler (e.g. gcc), autotools (which provides
"configure") and make (to run the Makefile) are needed to facilitate this.

If there was a git repository for the code, then a git submodule could be used
here rather than copying the source code.

At the time of writing, the source code here is version 3.49.

To update the source code, the "cfitsio" directory's contents should be replaced
with the new tarball's contents, and the docs directory within should also be
removed to help keep the size of the rust-fitsio git repo down.

## Patches

Any patches that are to be made against the source code are kept in the
`patches` subdirectory. These are to be applied at the root level of the
project after unpacking.
