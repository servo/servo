{

  description = "Servo web engine dev environment";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShells = {
        default = import ./shell.nix { buildAndroid = false; };
        android  = import ./shell.nix { buildAndroid = true; };
      };
    });

}
