{
  inputs = {
    dfinity-sdk = {
      url = "github:paulyoung/nixpkgs-dfinity-sdk";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs/21.11";
  };

  outputs = {
    self,
    nixpkgs,
    dfinity-sdk,
    flake-utils,
    mozillapkgs,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (final: prev: (import dfinity-sdk) final prev)
          ];
        };

        # Get a specific rust version
        mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") {};
        rust = (mozilla.rustChannelOf {
          channel = "stable";
          version = "1.54.0";
          sha256 = "NL+YHnOj1++1O7CAaQLijwAxKJW9SnHg8qsiOJ1m0Kk=";
          # sha256 = pkgs.lib.fakeSha256;
        }).rust.override {
          extensions = [
            "clippy-preview"
            # "miri-preview"
            # "rls-preview"
            # "rust-analyzer-preview"
            "rustfmt-preview"
            # "llvm-tools-preview"
            # "rust-analysis"
            # "rust-std"
            # "rustc-dev"
            # "rustc-docs"
            "rust-src"
          ];
          targets = [
            "wasm32-unknown-unknown"
          ];
        };

        # Override the version used in naersk
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };

        dfinitySdk = (pkgs.dfinity-sdk {
          acceptLicenseAgreement = true;
          sdkSystem =
            if system == "aarch64-darwin"
            then "x86_64-darwin"
            else system;
        })."0.8.4";
      in
        rec {
          # `nix build`
          packages.icfs = naersk-lib.buildPackage rec {
            pname = "icfs";
            root = ./.;
            buildInputs = [] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
            cargoBuildOptions = x: x ++ [
              "--package" pname
              "--target" "wasm32-unknown-unknown"
            ];
            cargoTestOptions = x: x ++ [
              "--package" pname
              "--target" "wasm32-unknown-unknown"
            ];
            compressTarget = false;
            copyBins = false;
            copyTarget = true;
          };

          defaultPackage = packages.icfs;

          # `nix develop`
          devShell = pkgs.mkShell {
            buildInputs = [
              dfinitySdk
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];

            # supply the specific rust version
            nativeBuildInputs = [ rust ];
          };
        }
    );
}