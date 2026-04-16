{ lib, pkgs, system }:
let
  nixosBinDir = lib.optionalString pkgs.stdenv.isLinux (
    lib.makeBinPath [
      (pkgs.buildFHSEnv {
        name = "ruff";
        runScript = pkgs.writeShellScript "ruff-fhs" ''
          exec "${toString ./.}/.venv/bin/ruff" "$@"
        '';
      })

      (pkgs.buildFHSEnv {
        name = "pyrefly";
        runScript = pkgs.writeShellScript "pyrefly-fhs" ''
          exec "${toString ./.}/.venv/bin/pyrefly" "$@"
        '';
      })
    ]
  );
in ''
  echo "Servo dev shell loaded for ${system}"
  # Fix invalid option errors during linking
  # https://github.com/mozilla/nixpkgs-mozilla/commit/c72ff151a3e25f14182569679ed4cd22ef352328
  unset AS
  # Compiling programs under Nix sets the interpreter (ELF INTERP) and rpath (ELF DT_RUNPATH [1])
  # to ensure that it can find the needed (ELF DT_NEEDED) libraries in the Nix store.
  #
  # This is good on NixOS, but bad everywhere else. Using the Nix interpreter makes the programs
  # dependent on the Nix store, making them impossible to distribute and run on other machines
  # without `nix bundle`. Even on the same machine, the program will crash in a variety of ways
  # because of the "OpenGL problem" [2] and other mismatches in X11 and Wayland libraries. Worse
  # still, it makes everyone else suffer the problems NixOS has, like needing $LD_LIBRARY_PATH
  # (see above) and needing to disable rust-lld (servo#30123).
  #
  # We can make the programs independent of Nix by resetting $NIX_DYNAMIC_LINKER to the system
  # interpreter, setting $NIX_DONT_SET_RPATH to prevent the clang and ld wrappers from adding
  # -rpath options to $NIX_LDFLAGS [3][4], and removing any -rpath options that get added by
  # clangStdenv despite $NIX_DONT_SET_RPATH.
  #
  # This is comparable to fixing target/*/servo after the fact with:
  #
  #     patchelf --remove-rpath --set-interpreter $(patchelf --print-interpreter /usr/bin/env)
  #
  # [1] DT_RPATH breaks LD_LIBRARY_PATH and is no longer used
  #     https://medium.com/obscure-system/rpath-vs-runpath-883029b17c45
  # [2] "Using Nix on non-NixOS distros, it's common to see GL application errors:"
  #     https://github.com/nix-community/nixGL
  # [3] https://ryantm.github.io/nixpkgs/stdenv/stdenv/#bintools-wrapper
  # [4] https://matklad.github.io/2022/03/14/rpath-or-why-lld-doesnt-work-on-nixos.html
  if ! [ -e /etc/NIXOS ]; then
    set -- $NIX_LDFLAGS
    for i; do
      shift
      if [ "$i" = -rpath ]; then
        shift
      else
        set -- "$@" "$i"
      fi
    done
    export NIX_DYNAMIC_LINKER=$(patchelf --print-interpreter /usr/bin/env)
    export NIX_DONT_SET_RPATH=1
    export NIX_LDFLAGS="$@"
    # Don't pollute ~/.rustup with toolchains installed by nixpkgs rustup, because they
    # get patched in a way that makes them dependent on the Nix store.
    repo_root=$(git rev-parse --show-toplevel)
    export RUSTUP_HOME=$repo_root/.rustup
  else
    # On NixOS, export FHS wrapper paths so mach can prepend them to PATH at runtime
    # This ensures the FHS-wrapped binaries take precedence over .venv/bin
    export SERVO_NIX_BIN_DIR="${nixosBinDir}"
  fi
''
