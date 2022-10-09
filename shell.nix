{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    rustup
    libiconv
    cfitsio
    pkg-config
  ];
}
