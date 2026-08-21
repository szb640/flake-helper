{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  in {
    packages.${system}.fh = pkgs.rustPlatform.buildRustPackage {
      pname = cargoConfig.package.name;
      version = cargoConfig.package.version;

      src = ./.;

      cargoHash = "sha256-EZWYwhT/uSGymIgTqTi0GoY9L6aCPukHRf5GIBIn4SA=";

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
