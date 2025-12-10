{
  description = "openmensa-parser-darmstadt";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
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
          openmensa-parser-darmstadt = pkgs.rustPlatform.buildRustPackage {
            inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name;
            inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) version;

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;

            # nativeBuildInputs = [
            #   pkgs.pkg-config
            # ];

            # buildInputs = [
            #   pkgs.openssl
            # ];
          };

          default = openmensa-parser-darmstadt;
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
