# Steps

1. Download version x of cfitsio
2. Compile source code
3. Install to unique prefix
4. Set `PKG_CONFIG_PATH` envar to include `prefix/lib/pkgconfig`
5. Compile `fitsio` crate
    * optionally check teh compilation output that the link flags are correct
6. Check for compile errors

## Questions

* How isolated does the system have to be?
    * do we need full isolation e.g. docker containers?
    * how are we going to test the compiling of the crate?
        * download the repo via git and compile?
        * compile using the main repo?
        * create a stub project that just includes the dependency?
* If we use docker, what steps do we need to do?
    * Need to install pkg-config and c build system
* What should be in the dockerfile and what should be part of the run command?
* Do we make one docker image per cfitsio version?
* What do we test?
    * `fitsio`
    * `fitsio` with `bindgen` feature
