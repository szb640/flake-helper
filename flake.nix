{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.${system}.fh = pkgs.rustPlatform.buildRustPackage {
      pname = "fh";
      version = "0.1";

      src = ./.;

      cargoHash = "sha256-+cmIbZBmSSemXzmK6i2a0uOK+dDfvB3LFrRl3cZZf1w=";

      meta = {
        description = "Utilities for managing development environments with flake.";
        mainProgram = "fh";
        maintainers = [{ name = "szb640"; }];
      };
    };
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs;[
        rustc
        cargo
        rustfmt
        clippy
      ];
    };
  };
}
