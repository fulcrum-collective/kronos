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
      in
      {
        devShells = {
          build = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rustc
              rustfmt
            ];
          };

          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rustc
              rustfmt
            ];
          };

          fuzzing = pkgs.mkShell {
            packages = with pkgs; [
              rustup
            ];

            shellHook = ''
              if ! command -v rustc >/dev/null; then
                echo "Rust not found, installing via rustup..."
                rustup install nightly
                cargo install cargo-fuzz
              fi
              rustup default nightly
              echo "Rust version:"
              rustc --version
            '';
          };

          test = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rustc
              rustfmt
            ];
          };
        };
      });
}
