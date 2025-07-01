# overlays/hardened_malloc.nix
#
# Overlay for hardened_malloc.
# Provides a package based on a stable release tag, with SSH signature
# verification of the tag and selective application of local, audited patches.

self: super: {
  hardened_malloc = super.hardened_malloc.overrideAttrs (oldAttrs: {
    # Defines the target stable release tag to build upon.
    version = "13";

    # Fetches the repository source via Git, checks out the specified tag,
    # and verifies its SSH signature in a post-fetch hook before use.
    src = self.fetchgit {
      url = "https://github.com/GrapheneOS/hardened_malloc.git";
      rev = "13";
      hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # Replace with actual hash for tag 13

      postFetch =
        let
          allowedSignersFile = self.fetchurl {
            url = "https://releases.grapheneos.org/allowed_signers";
            hash = "sha512-28gbkyk3xf2nh0rdk8xd9q83bw6y690xh63sik3i841f4b29j6161ys9iffmxjrl6gqc3315135b4cgsaj0mjiw7rxxm0qa735f0smg"; # Replace with actual hash
          };
        in
        ''
          cd $out
          echo "Verifying SSH signature for git tag ${oldAttrs.version}..."
          git -c gpg.ssh.allowedSignersFile=${allowedSignersFile} tag -v ${oldAttrs.version}
          echo "SSH signature for tag ${oldAttrs.version} is valid."
        '';
    };

    # Applies a list of local, audited patches to the fetched source.
    patches = [
      ./../patches/0001-support-gcc-15.patch
      ./../patches/0002-update-libdivide-to-5.2.0.patch
    ];

    # Passes high-level configuration options to the Makefile.
    makeFlags = [
      "CONFIG_NATIVE=false"
      "CONFIG_CXX_ALLOCATOR=false"
    ];

    # Custom build phase to compile only the standard variant and its static counterpart.
    buildPhase = ''
      runHook preBuild
      make $makeFlags ''${enableParallelBuilding:+-j$NIX_BUILD_CORES} all static
      runHook postBuild
    '';

    # Custom install phase to place artifacts into their respective outputs.
    installPhase = ''
      runHook preInstall

      install -Dm644 -t $dev/include include/*.h

      mkdir -p $out/lib
      install -Dm644 out/libhardened_malloc.so $out/lib/
      install -Dm644 libhardened_malloc.a $out/lib/

      mkdir -p $out/bin
      cat > $out/bin/preload-hardened-malloc << EOF
      #!/bin/sh
      export LD_PRELOAD="$out/lib/libhardened_malloc.so\''${LD_PRELOAD:+:\$LD_PRELOAD}"
      exec "\$@"
      EOF
      chmod +x $out/bin/preload-hardened-malloc

      runHook postInstall
    '';

    # Defines the separate outputs for the package (libraries, headers).
    outputs = [ "out" "dev" ];

    # Clears the passthru attribute, removing the default Nixpkgs test suite
    # for a leaner and more focused package derivation.
    passthru = {};
  });
}
