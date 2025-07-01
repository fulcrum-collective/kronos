# flake.nix
{
  description = "A hardened, statically-linked Rust binary for Kronos";

  # Defines the project's external dependencies (flakes).
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  # Defines what this flake provides (packages, apps, shells).
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # 1. Compose all overlays into a single list.
        # This is where we apply our customizations to the Nix package set.
        myOverlays = [
          (import ./overlays/hardened-musl.nix)
          (import ./overlays/hardened_malloc.nix)
          rust-overlay.overlays.default
        ];

        # 2. Import nixpkgs, applying our custom overlays.
        pkgs = import nixpkgs {
          inherit system;
          overlays = myOverlays;
        };

        # Create a Clang-based package set, also with our overlays.
        pkgs_clang = import nixpkgs {
          inherit system;
          overlays = myOverlays;
          stdenv = pkgs.clangStdenv;
        };

        # Select the desired Rust toolchain from the overlaid package set.
        rustToolchain = pkgs.rust-bin.stable.latest.default;

      in
      {
        # Development shell for working on the project.
        devShells.default = pkgs.mkShell {
          # Provides development tools.
          packages = with pkgs; [
            cargo-watch
            clippy
          ];

          # Pulls in the build dependencies from the kronos package itself.
          inputsFrom = [
            self.packages.${system}.kronos
          ];
        };

        # The final build definition for the kronos package.
        packages.kronos = rustToolchain.rustPlatform.buildRustPackage {
          pname = "kronos";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          # 3. Define build inputs using our overlaid, customized packages.
          nativeBuildInputs = [
            pkgs_clang.clang
            # These are now the custom, hardened versions defined by our overlays.
            pkgs_clang.musl
            pkgs_clang.hardened_malloc
          ];

          # Sets the build target for a static musl binary.
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

          # Specifies Clang as the linker for this specific target.
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs_clang.clang}/bin/clang";

          # 4. Unifies the hardening strategy for the final link stage.
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = [
            # Add the path to our custom hardened_malloc library.
            "-L" "native=${pkgs_clang.hardened_malloc}/lib"
            # The build.rs script will tell rustc to link this library.
          ] ++ (
            # Inherit all hardening flags from our custom musl build.
            # This ensures the final executable is linked with the same security policies.
            map (flag: "-C" + "link-arg=" + flag) (pkgs_clang.musl.CFLAGS ++ pkgs_clang.musl.LDFLAGS)
          );
        };

        # Defines the default package for `nix build`.
        packages.default = self.packages.${system}.kronos;

        # Defines the default app for `nix run`.
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.kronos;
        };
      }
    );
}
