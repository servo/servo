# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import shutil
import subprocess
from typing import Any, Optional

import distro

from .base import Base
from .build_target import BuildTarget


def parse_pkg_file(filename: str) -> list[str]:
    """Parse a file containing a list of packages, one per line, filtering out comments."""
    with open(filename, "r") as f:
        lines = [line.strip() for line in f.read().splitlines()]
        packages = [line for line in lines if not line.startswith("#")]
        return packages


def apt_packages() -> list[str]:
    basepath = os.path.dirname(__file__)
    apt_pkgs: list[str] = []
    for file in os.listdir(os.path.join(basepath, "linux_packages", "apt")):
        # Note: For now we also include ubuntu-only packages, since not available packages
        # will be filtered out automatically, since we check which packages are available.
        if file.endswith(".txt") and file.startswith("apt_"):
            filepath = os.path.join(basepath, "linux_packages", "apt", file)
            apt_pkgs.extend(parse_pkg_file(filepath))
    if len(apt_pkgs) == 0:
        raise RuntimeError("No apt packages found.")
    return apt_pkgs


def dnf_packages() -> list[str]:
    basepath = os.path.dirname(__file__)
    dnf_pkgs: list[str] = []
    for file in os.listdir(os.path.join(basepath, "linux_packages", "dnf")):
        if file.endswith(".txt") and file.startswith("dnf_"):
            filepath = os.path.join(basepath, "linux_packages", "dnf", file)
            dnf_pkgs.extend(parse_pkg_file(filepath))
    if len(dnf_pkgs) == 0:
        raise RuntimeError("No dnf packages found.")
    return dnf_pkgs


def xbps_packages() -> list[str]:
    basepath = os.path.dirname(__file__)
    pkgs: list[str] = []
    for file in os.listdir(os.path.join(basepath, "linux_packages", "xbps")):
        if file.endswith(".txt") and file.startswith("xbps_"):
            filepath = os.path.join(basepath, "linux_packages", "xbps", file)
            pkgs.extend(parse_pkg_file(filepath))
    if len(pkgs) == 0:
        raise RuntimeError("No xbps packages found.")
    return pkgs


class Linux(Base):
    def __init__(self, *args: str, **kwargs: Any) -> None:
        super().__init__(*args, **kwargs)
        self.is_linux = True
        self.distro = distro.name()

    def _platform_bootstrap(self, force: bool, yes: bool) -> bool:
        if self.distro.lower() == "nixos":
            print("NixOS does not need bootstrap, it will automatically enter a nix-shell")
            print("Just run ./mach build")
            print("")
            print("You will need to run a nix-shell if you are trying to run any of the built binaries")
            print("To enter the nix-shell manually use:")
            print("  $ nix-shell")
            return False

        if self.distro.lower() not in [
            "arch linux",
            "arch",
            "artix",
            "endeavouros",
            "centos linux",
            "centos",
            "debian gnu/linux",
            "raspbian gnu/linux",
            "fedora linux",
            "fedora",
            "nixos",
            "ubuntu",
            "linuxmint",
            "linux mint",
            "kde neon",
            "pop!_os",
            "tuxedo os",
            "elementary",
            "void",
            "fedora linux asahi remix",
        ]:
            print(
                f"mach bootstrap does not support {self.distro}."
                " You may be able to install dependencies manually."
                " See https://github.com/servo/servo/wiki/Building."
            )
            input("Press Enter to continue...")
            return False

        installed_something = self.install_non_gstreamer_dependencies(force, yes)
        return installed_something

    def install_non_gstreamer_dependencies(self, force: bool, yes: bool = False) -> bool:
        def check_sudo() -> bool:
            if os.geteuid() != 0:  # pyrefly: ignore[missing-attribute]
                if shutil.which("sudo") is None:
                    return False
            return True

        def run_as_root(command: list[str], force: bool = False, yes: bool = False) -> int:
            if os.geteuid() != 0:  # pyrefly: ignore[missing-attribute]
                command.insert(0, "sudo")
            if force or yes:
                command.append("-y")
            return subprocess.call(command)

        install = False
        pkgs = []
        command = []
        if self.distro in ["Ubuntu", "Debian GNU/Linux", "Raspbian GNU/Linux"]:
            command = ["apt-get", "install", "-m"]
            pkgs = apt_packages()

            # Skip 'clang' if 'clang' binary already exists.
            result = subprocess.run(["which", "clang"], capture_output=True)
            if result and result.returncode == 0:
                pkgs.remove("clang")

            # Try to filter out unknown packages from the list. This is important for Debian
            # as it does not ship all of the packages we want.
            # We need to run 'apt-get update' first to make sure the package cache is populated.
            run_as_root(["apt-get", "update"])
            installable = subprocess.check_output(["apt-cache", "--generate", "pkgnames"])
            if installable:
                installable = installable.decode("ascii").splitlines()
                missing_pkgs = list(filter(lambda pkg: pkg not in installable, pkgs))
                pkgs = list(filter(lambda pkg: pkg in installable, pkgs))
                if len(missing_pkgs) > 0:
                    print(
                        "Skipping the following required packages, as they don't exist in this OS version:",
                        missing_pkgs,
                    )

            if subprocess.call(["dpkg", "-s"] + pkgs, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE) != 0:
                install = True
        elif self.distro in ["CentOS", "CentOS Linux", "Fedora", "Fedora Linux", "Fedora Linux Asahi Remix"]:
            command = ["dnf", "install"]
            installed_pkgs: list[str] = subprocess.check_output(
                ["rpm", "--query", "--all", "--queryformat", "%{NAME}\n"], encoding="utf-8"
            ).splitlines()
            pkgs = dnf_packages()
            for pkg in pkgs:
                if pkg not in installed_pkgs:
                    install = True
                    break
        elif self.distro == "void":
            command = ["xbps-install", "-A"]
            installed_pkgs = subprocess.check_output(["xbps-query", "-l"], text=True).splitlines()
            pkgs = xbps_packages()
            for pkg in pkgs:
                if "ii {}-".format(pkg) not in installed_pkgs:
                    install = force = True
                    break

        if not install:
            return False

        print("Installing missing dependencies...")
        if not check_sudo():
            print(
                "'sudo' command not found."
                " You may be able to install dependencies manually."
                " See https://github.com/servo/servo/wiki/Building."
            )
            input("Press Enter to continue...")
            return False

        if run_as_root(command + pkgs, force, yes) != 0:
            raise EnvironmentError("Installation of dependencies failed.")
        return True

    def gstreamer_root(self, target: BuildTarget) -> Optional[str]:
        return None

    def _platform_bootstrap_gstreamer(self, target: BuildTarget, force: bool) -> bool:
        raise EnvironmentError(
            "Bootstrapping GStreamer on Linux is not supported. "
            + "Please install it using your distribution package manager."
        )
