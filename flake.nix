{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
        };
        devShell = with pkgs; mkShell {
          buildInputs = [ pkg-config openssl.dev cargo rustc rustfmt pre-commit rustPackages.clippy ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/carbonex";
        };
        nixosModules.carbonIntensityExporter = { config, lib, ...}: {
          options.services.carbonIntensityExporter = with lib; {
            enable = lib.mkEnableOption "enable carbon intensity exporter";
            listenAddr = lib.mkOption {
              type = types.str;
              default = ":38389";
            };
          };
          config = lib.mkIf config.services.carbonIntensityExporter.enable {
            systemd.services.carbon-intensity-exporter = {
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                User = "carboninex";
                Group = "carboninex";
                DynamicUser = true;
                Restart = "always";
                ExecStart = "${self.packages."${system}".default}/bin/cmd";
              };
            };
          };
        };
      });
}