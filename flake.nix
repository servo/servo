{

  description = "Servo web engine dev environment";

  inputs = {
    nixpkgs.url = "github:nixOS/nixpkgs/ffbc9f8cbaacfb331b6017d5a5abb21a492c9a38";
    nixpkgs-gnumake.url = "github:nixOS/nixpkgs/6adf48f53d819a7b6e15672817fa1e78e5f4e84f";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, nixpkgs-gnumake, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in {

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ];
          };

          androidPkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ];
            config = {
              allowUnfree = true;
              android_sdk.accept_license = true;
            };
          };

          deps = import ./nix/packages.nix {
            inherit pkgs;
            inherit (pkgs) lib stdenv;
            gnumakeSource = nixpkgs-gnumake.legacyPackages.${system};
          };

          androidDeps = import ./nix/android.nix {
            inherit androidPkgs;
          };

          hook = import ./nix/shell-hook.nix {
            inherit pkgs system;
            inherit (pkgs) lib;
          };
        in {

          default = pkgs.mkShell ({
            buildInputs = deps.buildInputs;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps.runtimeLibs;

            shellHook = hook;
          } // deps.envVars);

          android = pkgs.mkShell ({
            buildInputs = deps.buildInputs ++ androidDeps.buildInputs;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps.runtimeLibs;

            shellHook = hook;
          } // deps.envVars // androidDeps.envVars);

        }
      );

    };

}
