{
  description = "A development shell for the Kronos project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.rustup ];

          shellHook = ''
            if ! command -v rustc >/dev/null; then
              echo "Rust not found, installing via rustup..."
              rustup install stable
            fi
            rustup default stable
            echo "Rust version:"
            rustc --version
          '';
        };
      }
    );
}
