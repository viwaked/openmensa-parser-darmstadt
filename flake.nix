{
  description = "openmensa-parser-darmstadt";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      mkPkgs =
        system:
        import nixpkgs {
          inherit system;
        };
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "x86_64-darwin"
          "aarch64-linux"
          "aarch64-darwin"
        ] (system: function system);
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
        in
        rec {
          openmensa-parser-darmstadt-server = pkgs.rustPlatform.buildRustPackage {
            inherit ((builtins.fromTOML (builtins.readFile ./server/Cargo.toml)).package) name;
            inherit ((builtins.fromTOML (builtins.readFile ./server/Cargo.toml)).package) version;

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;

            buildAndTestSubdir = "server";

            meta.mainProgram = "openmensa-parser-darmstadt-server";
          };

          openmensa-parser-darmstadt-cli = pkgs.rustPlatform.buildRustPackage {
            inherit ((builtins.fromTOML (builtins.readFile ./cli/Cargo.toml)).package) name;
            inherit ((builtins.fromTOML (builtins.readFile ./cli/Cargo.toml)).package) version;

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;

            buildAndTestSubdir = "cli";

            meta.mainProgram = "openmensa-parser-darmstadt-cli";
          };

          server = openmensa-parser-darmstadt-server;
          cli = openmensa-parser-darmstadt-cli;
          default = cli;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          f =
            with fenix.packages.${system};
            combine [
              latest.toolchain
            ];
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              f
              cargo-watch
              openssl
              pkg-config
            ];

            RUST_BACKTRACE = 1;
          };
        }
      );
    };
}
