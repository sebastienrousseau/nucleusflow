{
  description = "High-performance, memory-safe static site generator written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nucleusflow";
          version = "0.0.10";

          src = ./../..;

          cargoLock = {
            lockFile = ./../../Cargo.lock;
          };

          postInstall = ''
            make -f GNUmakefile PREFIX=$out install
          '';

          meta = with pkgs.lib; {
            description = "High-performance, memory-safe static site generator written in Rust";
            homepage = "https://nucleusflow.com";
            license = with licenses; [ asl20 mit ];
            maintainers = [ ];
          };
        };
      });
}
