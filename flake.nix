{
  description = "A development shell for the Kronos project";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        # Defines a standard development environment.
        devShells.default = pkgs.mkShell {
          # Provides the necessary tools for development.
          packages = with pkgs; [
            # A stable Rust toolchain
            stable.rust-toolchain
            # Additional useful tools
            cargo-watch
            clippy
          ];
        };
      });
}
