{
  description = "Flake utils demo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell rec {
          buildInputs = [
            pkgs.rustup
            pkgs.libiconv
            pkgs.automake
            pkgs.autoconf
            pkgs.cfitsio
            pkgs.bzip2
            pkgs.pkg-config
            pkgs.zlib
            pkgs.cargo-release
            pkgs.cargo-nextest
            pkgs.bacon
            # for bin/test
            pkgs.python3
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.cargo-tarpaulin
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          shellHook = ''
            export RUST_BUILD_BASE="$HOME/.cache/rust-builds"
            WORKSPACE_ROOT=$(cargo metadata --no-deps --offline 2>/dev/null | jq -r ".workspace_root")
            PACKAGE_BASENAME=$(basename $WORKSPACE_ROOT)
            # Run cargo with target set to $RUST_BUILD_BASE/$PACKAGE_BASENAME
            export CARGO_TARGET_DIR="$RUST_BUILD_BASE/$PACKAGE_BASENAME"

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
