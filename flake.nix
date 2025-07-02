{
  description = "A hardened, statically-linked Rust binary for Kronos";

  # Defines the project's external dependencies (flakes).
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let

      flakeRoot = builtins.path { path = ./.; name = "flake-root"; };
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Compose all overlays into a single list.
        myOverlays = [
          (import ./overlays/hardened-musl.nix)
          (import ./overlays/hardened_malloc.nix { inherit flakeRoot; })
          rust-overlay.overlays.default
        ];

        # Apply overlays to nixpkgs
        pkgs = import nixpkgs {
          inherit system;
          overlays = myOverlays;
        };

        pkgs_clang = import nixpkgs {
          inherit system;
          overlays = myOverlays;
          stdenv = pkgs.clangStdenv;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo-watch
            clippy
          ];

          inputsFrom = [
            self.packages.${system}.kronos
          ];
        };

        packages.kronos = pkgs.rustPlatform.buildRustPackage {
          pname = "kronos";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs_clang.clang
            pkgs_clang.musl
            pkgs_clang.hardened_malloc
          ];

          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs_clang.clang}/bin/clang";

          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = [
            "-L" "native=${pkgs_clang.hardened_malloc}/lib"
          ] ++ (
            map (flag: "-C" + "link-arg=" + flag) (pkgs_clang.musl.CFLAGS ++ pkgs_clang.musl.LDFLAGS)
          );

          RUSTC = rustToolchain;
          CARGO = rustToolchain;
        };

        packages.default = self.packages.${system}.kronos;

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.kronos;
        };
      }
    );
}
