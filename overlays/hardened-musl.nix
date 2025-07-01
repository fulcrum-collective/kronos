# overlays/hardened-musl.nix
self: super: {
  musl = super.musl.overrideAttrs (oldAttrs:
    let
      # A derivation to PGP-verify the source tarball before use, ensuring supply chain integrity.
      verified_src =
        let
          musl-src = self.fetchurl {
            url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz";
            hash = "sha512-1yc2n707vkimm3zix6zz72a1ykx2lq6c5dkz2n8q5fdf1dyl5l8mpw7b7j87ma3rlfhddiycs4m5zabg3qsbf59l73nkk93761zgdvv";
          };
          musl-sig = self.fetchurl {
            url = "https://musl.libc.org/releases/musl-1.2.5.tar.gz.asc";
            hash = "sha512-0lr478mn3rdfq5dsdxzs72aiv2ll81194d3c6k8r6qpnslr9h37lsn4zwb5bwb41gh8bjxird9nz7mjv3figjk8ylzx7fsan42vzbn8";
          };
          musl-pubkey = self.fetchurl {
            url = "https://musl.libc.org/musl.pub";
            hash = "sha512-20sd9jrmbp3gxx6aj3jf0nsx7hfshfhwrghd4dxjdp9vs4vzbpf5lpxla3s5j8vv9qk7rvv0r4haqkghlqlhhncagl6zs5g0mf3ljgm";
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
      version = "1.2.5";

      # Use the PGP-verified source.
      src = verified_src;

      # Append our hardening flags to any flags already present in the original derivation.
      CFLAGS = oldAttrs.CFLAGS ++ hardeningFlags;
      CXXFLAGS = oldAttrs.CXXFLAGS ++ hardeningFlags;
      LDFLAGS = (oldAttrs.LDFLAGS or []) ++ hardeningLDFLAGS;
    });
}
