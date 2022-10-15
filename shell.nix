{ pkgs ? import <nixpkgs> { } }:
with pkgs;
pkgs.mkShell rec {
  buildInputs = [
    rustup
    libiconv
    cfitsio
    pkg-config
    # for bin/test
    python3
  ];
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
}
