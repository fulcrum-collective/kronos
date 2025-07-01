# overlays/hardened-musl.nix
self: super: {
  musl = super.musl.overrideAttrs (oldAttrs:
    let
      # A derivation to PGP-verify the source tarball before use, ensuring supply chain integrity.
      verified_src =
        let
          musl-src = self.fetchurl {
            url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz";
            sha256 = "sha256-qaEYu+hNh2TaDqDSizqz+uhHf8fkCF2QECuFlvx8deQ=";
          };
          musl-sig = self.fetchurl {
            url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz.asc";
            sha256 = "sha256-rE7i3zV4l33p3mBq7iSgJcA/Kz4dJqU33lXXu2VAWmo=";
          };
          musl-pubkey = self.fetchurl {
            url = "https://musl.libc.org/musl.pub";
            sha256 = "sha256-vSVsPZzzmYjT4aWvzsRk8n4e55oTQCzLckD1g2b5GfE=";
          };
        in
        self.stdenv.mkDerivation {
          pname = "musl-verified-src";
          version = oldAttrs.version;
          srcs = [ musl-src musl-sig musl-pubkey ];
          sourceRoot = ".";
          nativeBuildInputs = [ self.gnupg ];
          unpackPhase = ''
            export GNUPGHOME=$(mktemp -d)
            gpg --import ${musl-pubkey}
            echo "Verifying PGP signature for musl source..."
            gpgv ${musl-sig} ${musl-src}
            echo "Signature OK. Unpacking source..."
            tar -xzf ${musl-src}
          '';
          dontBuild = true;
          dontInstall = true;
        };

      # --- Hardening Flag Sets ---
      # This configuration enables a comprehensive suite of modern, software-based
      # exploit mitigations that are portable across CPU architectures.

      hardeningFlags = [
        # Harden against format string vulnerabilities.
        "-Wformat=2"
        "-Werror=format-security"

        # Memory, stack, and source code protection.
        "-fstack-clash-protection"
        "-fsanitize=safe-stack"
        "-D_FORTIFY_SOURCE=3"
        "-fno-common"
        "-fno-plt"

        # Link Time Optimization (LTO) and Control-Flow Integrity (CFI).
        "-flto=thin"
        "-fsanitize=cfi"
        "-fvisibility=hidden"

        # Undefined Behavior Sanitizer (UBSan) to catch integer overflows and more.
        "-fsanitize=undefined"
      ];

      # Corresponding flags for the linker.
      hardeningLDFLAGS = [
        "-Wl,-z,relro,-z,now" # Enable Full RELRO for ELF hardening.
        "-flto=thin"
        "-fsanitize=cfi"
        "-fsanitize=undefined"
      ];

    in
    {
      # Use the PGP-verified source.
      src = verified_src;

      # Append our hardening flags to any flags already present in the original derivation.
      CFLAGS = oldAttrs.CFLAGS ++ hardeningFlags;
      CXXFLAGS = oldAttrs.CXXFLAGS ++ hardeningFlags;
      LDFLAGS = (oldAttrs.LDFLAGS or []) ++ hardeningLDFLAGS;
    });
}
