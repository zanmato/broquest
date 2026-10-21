{
  description = "Broquest, a desktop API client";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    # The vendor/llrt submodule, at the commit it pins. Fetching it with the
    # flake's own submodules would recurse into llrt's, whose web platform
    # tests take gigabytes, so it is an input of its own. Move this with the
    # submodule. The release workflow fails when the two differ.
    llrt = {
      url = "github:awslabs/llrt/488ebe1c175df54577db07525490a5dca076538b";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      llrt,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = function: nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});

      # Built from the package set it is given, so the overlay links against
      # the system's own glibc, which its graphics drivers were built with.
      broquestFor = pkgs: pkgs.callPackage ./nix/package.nix {
          craneLib = crane.mkLib pkgs;
          inherit llrt;
        };
    in
    {
      packages = forEachSystem (pkgs: rec {
        broquest = broquestFor pkgs;
        default = broquest;
      });

      overlays.default = final: prev: { broquest = broquestFor final; };

      devShells = forEachSystem (
        pkgs:
        let
          broquest = self.packages.${pkgs.stdenv.hostPlatform.system}.broquest;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ broquest ];
            packages = with pkgs; [
              cargo
              clippy
              rustc
              rustfmt
              rust-analyzer
            ];
            # cargo run starts Broquest from target/, which has no rpath to the
            # libraries GPUI loads at runtime.
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath broquest.runtimeLibraries;
          };
        }
      );

      formatter = forEachSystem (pkgs: pkgs.nixfmt);
    };
}
