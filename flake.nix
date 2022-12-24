{
  description = "Flake utils demo";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustup
            pkgs.libiconv
            pkgs.cfitsio
            pkgs.pkg-config
            # for bin/test
            pkgs.python3
          ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
