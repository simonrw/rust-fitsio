{
  description = "Flake utils demo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        python-overlay = self: super: {
          python310 = super.python310.override {
            packageOverrides = pyself: pysuper: {
              fitsio = pysuper.buildPythonPackage rec {
                pname = "fitsio";
                version = "1.1.8";
                src = pysuper.fetchPypi {
                  inherit pname version;
                  hash = "sha256-YfVpsmgqDK3OUsllPwybgflR0ABSLO9kXOHLSfeDAPk=";
                };
                propagatedBuildInputs = [
                  pyself.setuptools
                  pyself.ipython
                ];
                buildInputs = [
                  pyself.numpy
                  self.pkg-config
                  self.bzip2
                ];
                MAKEFLAGS = "-j";
              };
            };
          };
        };

        overlays = [
          rust-overlay.overlays.default
          python-overlay
        ];
        pkgs = import nixpkgs {
          inherit overlays system;
        };
      in
      {
        devShells.default = pkgs.mkShell rec {
          buildInputs = [
            pkgs.rust-bin.beta.latest.default
            pkgs.clippy
            pkgs.rustfmt
            pkgs.libiconv
            pkgs.cfitsio
            pkgs.bzip2
            pkgs.pkg-config
            pkgs.cargo-release
            pkgs.rust-analyzer
            # for bin/test
            (pkgs.python310.withPackages
              (ps: with ps; [
                numpy
                fitsio
              ]))
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.cargo-tarpaulin
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          shellHook = ''
            # From: https://github.com/NixOS/nixpkgs/blob/1fab95f5190d087e66a3502481e34e15d62090aa/pkgs/applications/networking/browsers/firefox/common.nix#L247-L253
            # Set C flags for Rust's bindgen program. Unlike ordinary C
            # compilation, bindgen does not invoke $CC directly. Instead it
            # uses LLVM's libclang. To make sure all necessary flags are
            # included we need to look in a few places.
            #
            # source: https://hoverbear.org/blog/rust-bindgen-in-nix/
            export BINDGEN_EXTRA_CLANG_ARGS="$(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags) \
              ${pkgs.lib.optionalString pkgs.stdenv.cc.isClang "-idirafter ${pkgs.stdenv.cc.cc}/lib/clang/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include"} \
              ${pkgs.lib.optionalString pkgs.stdenv.cc.isGNU "-isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc} -isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/${pkgs.stdenv.hostPlatform.config} -idirafter ${pkgs.stdenv.cc.cc}/lib/gcc/${pkgs.stdenv.hostPlatform.config}/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include"} \
              "
          '';
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
