{ flakeRoot }:

self: super: {
  hardened_malloc = super.graphene-hardened-malloc.overrideAttrs (oldAttrs: {
    version = "13";

    src = self.fetchgit {
      url = "https://github.com/GrapheneOS/hardened_malloc.git";
      rev = "13";
      hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

      postFetch =
        let
          allowedSignersFile = self.fetchurl {
            url = "https://releases.grapheneos.org/allowed_signers";
            hash = "sha512-r2rgyjgKg9p7PjzKCqTSj5FVRihhDIafoVn26lxM2gcTjExiERcIimNGPQzsIBlv+BoIp9bRbBlAK1wf08/1kA==";
          };
        in
        ''
          cd $out
          echo "Verifying SSH signature for git tag ${oldAttrs.version}..."
          git -c gpg.ssh.allowedSignersFile=${allowedSignersFile} tag -v ${oldAttrs.version}
          echo "SSH signature for tag ${oldAttrs.version} is valid."
        '';
    };


    patches = [
      "${flakeRoot}/patches/0001-support-gcc-15.patch"
      "${flakeRoot}/patches/0002-update-libdivide-to-5.2.0.patch"
    ];

    makeFlags = [
      "CONFIG_NATIVE=false"
      "CONFIG_CXX_ALLOCATOR=false"
    ];

    buildPhase = ''
      runHook preBuild
      make $makeFlags ''${enableParallelBuilding:+-j$NIX_BUILD_CORES} all static
      runHook postBuild
    '';

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

    outputs = [ "out" "dev" ];
    passthru = {};
  });
}

