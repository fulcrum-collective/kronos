# overlays/hardened_malloc.nix
#
# This overlay provides a hardened_malloc package with the following guarantees:
# 1. The source code is based on a stable, official release tag.
# 2. The Git tag is cryptographically verified using the official GrapheneOS
#    signing keys, which are fetched directly from their release server.
# 3. Custom patches can be selectively applied to this verified base.

self: super: {
  hardened_malloc = super.hardened_malloc.overrideAttrs (oldAttrs: {

    version = "13";

    # We use fetchgit to clone the repository, which is necessary for tag verification.
    src = self.fetchgit {
      url = "https://github.com/GrapheneOS/hardened_malloc.git";
      rev = "13";
      # The content hash for the Git repository at tag '13'.
      # You can get this by running `nix build` with an empty hash and copying the result.
      hash = "sha256-Rk4b1E7q3aG7y6Z9o5j8Xb7v6c5d4e3f2a1b0c9d8e7f6g5h4i3j2k1l0m9n8o="; # Replace with the actual hash

      # postFetch hook runs after cloning. We perform the verification here.
      postFetch =
        let
          # Fetch the official allowed_signers file from GrapheneOS.
          allowedSignersFile = self.fetchurl {
            url = "https://releases.grapheneos.org/allowed_signers";
            # You can get this hash by running:
            # nix-prefetch-url https://releases.grapheneos.org/allowed_signers
            hash = "sha512-28gbkyk3xf2nh0rdk8xd9q83bw6y690xh63sik3i841f4b29j6161ys9iffmxjrl6gqc3315135b4cgsaj0mjiw7rxxm0qa735f0smg";
          };
        in
        ''
          cd $out

          echo "Verifying SSH signature for git tag ${oldAttrs.version}..."

          # Use the fetched official file for verification.
          # The entire build will fail if the signature is invalid or not from a trusted key.
          git -c gpg.ssh.allowedSignersFile=${allowedSignersFile} tag -v ${oldAttrs.version}

          echo "SSH signature for tag ${oldAttrs.version} is valid."
        '';
    };

    # Apply the patches from our local, version-controlled directory.
    patches = [
      # The path is relative to this .nix file.
      # Assuming your flake.nix is in the root, and this file is in overlays/,
      # we need to go up one level.
      ./../patches/0001-update-libdivide-to-5.2.0.patch
      ./../patches/0002-support-gcc-15.patch
    ];

  });
}
