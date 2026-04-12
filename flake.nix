{

  description = "Servo web engine dev environment";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:

  let
    allSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    forAllSystems = nixpkgs.lib.genAttrs allSystems;
  in {

    devShells = forAllSystems (system:
      let
        mkShell = buildAndroid: import ./shell.nix {
          inherit buildAndroid;
        };
      in {

        default = mkShell false;

      } // nixpkgs.lib.optionalAttrs (system != "aarch64-linux") {

        android = mkShell true;

      });
  };

}
