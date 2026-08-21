{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  in {
    packages.${system}.flake-helper = pkgs.rustPlatform.buildRustPackage {
      pname = cargoConfig.package.name;
      version = cargoConfig.package.version;

      src = ./.;

      cargoHash = "sha256-bvrjFR5prHS+rVz+TUBtBR4W8HFxPvVwzM8UWgJAn1U=";

      meta = {
        description = "Utilities for managing development environments with flake.";
        mainProgram = "fh";
        maintainers = [{ name = "szb640"; }];
      };
    };
    
    overlays.default = final: prev: {
      flake-helper = self.packages.${final.system}.flake-helper;
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
