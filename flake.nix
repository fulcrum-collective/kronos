# flake.nix
{
  description = "A hardened, statically-linked Rust binary for Kronos, built with Clang and a custom musl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    hardened_malloc_src.url = "github:GrapheneOS/hardened_malloc";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, hardened_malloc_src }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # --- 1. 組合所有 Overlays ---
        # 這是最關鍵的修改。我們將自訂的 overlay 和 rust-overlay 組合在一起。
        myOverlays = [
          (import ./overlays/hardened-musl.nix)
          rust-overlay.overlays.default
        ];

        # 導入 nixpkgs 並應用我們組合好的 overlays
        pkgs = import nixpkgs {
          inherit system;
          overlays = myOverlays;
        };

        # 使用 clangStdenv 來建立一個 Clang 的包集合
        pkgs_clang = import nixpkgs {
          inherit system;
          overlays = myOverlays; # 同樣應用 overlays
          stdenv = pkgs.clangStdenv;
        };

        # 選擇 Rust 工具鏈 (來自已經應用了 overlay 的 pkgs)
        rustToolchain = pkgs.rust-bin.stable.latest.default;

        # 編譯 hardened_malloc 的靜態庫 (使用 pkgs_clang 來確保 Clang 編譯)
        hardened-malloc-static = pkgs_clang.stdenv.mkDerivation {
          pname = "hardened_malloc-static";
          version = "11";
          src = hardened_malloc_src;
          makeFlags = [ "static" ];
          installPhase = ''
            runHook preInstall
            mkdir -p $out/lib
            cp libhardened_malloc.a $out/lib/
            runHook postInstall
          '';
        };

      in
      {
        # --- 開發環境 ---
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo-watch
            clippy
          ];

          inputsFrom = [
            self.packages.${system}.kronos
          ];
        };

        # --- 構建的軟體包 ---
        packages.kronos = rustToolchain.rustPlatform.buildRustPackage {
          pname = "kronos";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs_clang.clang
            # 這裡我們使用的 pkgs_clang.musl 已經是經過我們 overlay 硬化過的版本了！
            pkgs_clang.musl
            hardened-malloc-static
          ];

          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${pkgs_clang.clang}/bin/clang";

          # --- 2. 統一硬化策略 ---
          # 將 musl overlay 中的硬化旗標也應用到 kronos 的最終連結階段
          # 我們直接從硬化過的 musl 包中獲取這些旗標，確保一致性
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = [
            "-L" "native=${hardened-malloc-static}/lib"
            "-C" "link-arg=-lhardened_malloc"
          ] ++ (map (flag: "-C" + "link-arg=" + flag) (pkgs_clang.musl.CFLAGS ++ pkgs_clang.musl.LDFLAGS));
        };

        packages.default = self.packages.${system}.kronos;

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.kronos;
        };
      }
    );
}
