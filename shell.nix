# NOTE: This does not work offline or for nix-build
# For flake users: use `nix develop` instead
{ buildAndroid ? false }:
let
  pkgs = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/ffbc9f8cbaacfb331b6017d5a5abb21a492c9a38.tar.gz";
  }) {
    overlays = [
      (import (builtins.fetchTarball {
        url = "https://github.com/oxalica/rust-overlay/archive/99cc5667eece98bb35dcf35f7e511031a8b7a125.tar.gz";
      }))
    ];

    config = {
      android_sdk.accept_license = buildAndroid;
      allowUnfree = buildAndroid;
    };
  };

  deps = import ./nix/packages.nix {
    inherit pkgs;
    inherit (pkgs) lib;
  };

  androidDeps = if buildAndroid
    then import ./nix/android.nix { androidPkgs = pkgs; }
    else { buildInputs = []; envVars = {}; };

  hook = import ./nix/shell-hook.nix {
    inherit pkgs;
    inherit (pkgs) lib;
    system = pkgs.stdenv.hostPlatform.system;
  };
in pkgs.mkShell ({

  buildInputs = deps.buildInputs ++ androidDeps.buildInputs;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps.runtimeLibs;
  shellHook = hook;

} // deps.envVars // androidDeps.envVars)
