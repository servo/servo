# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import subprocess
from typing import Optional, Tuple

import distro
from .base import Base

# Please keep these in sync with the packages on the wiki, using the instructions below
# https://github.com/servo/servo/wiki/Building

# https://packages.debian.org
# https://packages.ubuntu.com
# 1. open devtools
# 2. paste in the whole APT_PKGS = [...]
# 3. copy(`sudo apt install ${APT_PKGS.join(" ")}`)
# 4. paste into https://github.com/servo/servo/wiki/Building#debian-based-distributions
APT_PKGS = [
    'build-essential', 'ccache', 'clang', 'cmake', 'curl', 'g++', 'git',
    'gperf', 'libdbus-1-dev', 'libfreetype6-dev', 'libgl1-mesa-dri',
    'libgles2-mesa-dev', 'libglib2.0-dev',
    'gstreamer1.0-plugins-good', 'libgstreamer-plugins-good1.0-dev',
    'gstreamer1.0-plugins-bad', 'libgstreamer-plugins-bad1.0-dev',
    'gstreamer1.0-plugins-ugly',
    "gstreamer1.0-plugins-base", 'libgstreamer-plugins-base1.0-dev',
    'gstreamer1.0-libav',
    'libgstrtspserver-1.0-dev',
    'gstreamer1.0-tools',
    'libges-1.0-dev',
    'libharfbuzz-dev', 'liblzma-dev', 'libudev-dev', 'libunwind-dev',
    'libvulkan1', 'libx11-dev', 'libxcb-render0-dev', 'libxcb-shape0-dev',
    'libxcb-xfixes0-dev', 'libxmu-dev', 'libxmu6', 'libegl1-mesa-dev',
    'llvm-dev', 'm4', 'xorg-dev', 'libxkbcommon0', "libxkbcommon-x11-0"
]

# https://packages.fedoraproject.org
# 1. open devtools
# 2. paste in the whole DNF_PKGS = [...]
# 3. copy(`sudo dnf install ${DNF_PKGS.join(" ")}`)
# 4. paste into https://github.com/servo/servo/wiki/Building#fedora
DNF_PKGS = ['libtool', 'gcc-c++', 'libXi-devel', 'freetype-devel',
            'libunwind-devel', 'mesa-libGL-devel', 'mesa-libEGL-devel',
            'glib2-devel', 'libX11-devel', 'libXrandr-devel', 'gperf',
            'fontconfig-devel', 'cabextract', 'ttmkfdir', 'expat-devel',
            'rpm-build', 'cmake', 'libXcursor-devel', 'libXmu-devel',
            'dbus-devel', 'ncurses-devel', 'harfbuzz-devel', 'ccache',
            'clang', 'clang-libs', 'llvm', 'python3-devel',
            'gstreamer1-devel', 'gstreamer1-plugins-base-devel',
            'gstreamer1-plugins-good', 'gstreamer1-plugins-bad-free-devel',
            'gstreamer1-plugins-ugly-free', 'libjpeg-turbo-devel',
            'zlib', 'libjpeg', 'vulkan-loader', 'libxkbcommon',
            'libxkbcommon-x11']

# https://voidlinux.org/packages/
# 1. open devtools
# 2. paste in the whole XBPS_PKGS = [...]
# 3. copy(`sudo xbps-install ${XBPS_PKGS.join(" ")}`)
# 4. paste into https://github.com/servo/servo/wiki/Building#void-linux
XBPS_PKGS = ['libtool', 'gcc', 'libXi-devel', 'freetype-devel',
             'libunwind-devel', 'MesaLib-devel', 'glib-devel', 'pkg-config',
             'libX11-devel', 'libXrandr-devel', 'gperf', 'bzip2-devel',
             'fontconfig-devel', 'cabextract', 'expat-devel', 'cmake',
             'cmake', 'libXcursor-devel', 'libXmu-devel', 'dbus-devel',
             'ncurses-devel', 'harfbuzz-devel', 'ccache', 'glu-devel',
             'clang', 'gstreamer1-devel', 'gst-plugins-base1-devel',
             'gst-plugins-good1', 'gst-plugins-bad1-devel',
             'gst-plugins-ugly1', 'vulkan-loader', 'libxkbcommon',
             'libxkbcommon-x11']

GSTREAMER_URL = \
    "https://github.com/servo/servo-build-deps/releases/download/linux/gstreamer-1.16-x86_64-linux-gnu.20190515.tar.gz"


class Linux(Base):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.is_linux = True
        (self.distro, self.version) = Linux.get_distro_and_version()

    @staticmethod
    def get_distro_and_version() -> Tuple[str, str]:
        distrib = distro.name()
        version = distro.version()

        if distrib in ['LinuxMint', 'Linux Mint', 'KDE neon', 'Pop!_OS', 'TUXEDO OS']:
            if '.' in version:
                major, _ = version.split('.', 1)
            else:
                major = version

            distrib = 'Ubuntu'
            if major == '22':
                version = '22.04'
            elif major == '21':
                version = '21.04'
            elif major == '20':
                version = '20.04'
            elif major == '19':
                version = '18.04'
            elif major == '18':
                version = '16.04'

        if distrib.lower() == 'elementary':
            distrib = 'Ubuntu'
            if version == '5.0':
                version = '18.04'
            elif version[0:3] == '0.4':
                version = '16.04'

        return (distrib, version)

    def _platform_bootstrap(self, force: bool) -> bool:
        if self.distro.lower() == 'nixos':
            print('NixOS does not need bootstrap, it will automatically enter a nix-shell')
            print('Just run ./mach build')
            print('')
            print('You will need to run a nix-shell if you are trying '
                  'to run any of the built binaries')
            print('To enter the nix-shell manually use:')
            print('  $ nix-shell etc/shell.nix')
            return False

        if self.distro.lower() == 'ubuntu' and self.version > '22.04':
            print(f"WARNING: unsupported version of {self.distro}: {self.version}")

        # FIXME: Better version checking for these distributions.
        if self.distro.lower() not in [
            'arch linux',
            'arch',
            'artix',
            'endeavouros',
            'centos linux',
            'centos',
            'debian gnu/linux',
            'raspbian gnu/linux',
            'fedora linux',
            'fedora',
            'nixos',
            'ubuntu',
            'void',
            'fedora linux asahi remix'
        ]:
            print(f"mach bootstrap does not support {self.distro}."
                  " You may be able to install dependencies manually."
                  " See https://github.com/servo/servo/wiki/Building.")
            input("Press Enter to continue...")
            return False

        installed_something = self.install_non_gstreamer_dependencies(force)
        return installed_something

    def install_non_gstreamer_dependencies(self, force: bool) -> bool:
        install = False
        pkgs = []
        if self.distro in ['Ubuntu', 'Debian GNU/Linux', 'Raspbian GNU/Linux']:
            command = ['apt-get', 'install', "-m"]
            pkgs = APT_PKGS

            # Skip 'clang' if 'clang' binary already exists.
            result = subprocess.run(['which', 'clang'], capture_output=True)
            if result and result.returncode == 0:
                pkgs.remove('clang')

            # Try to filter out unknown packages from the list. This is important for Debian
            # as it does not ship all of the packages we want.
            installable = subprocess.check_output(['apt-cache', '--generate', 'pkgnames'])
            if installable:
                installable = installable.decode("ascii").splitlines()
                pkgs = list(filter(lambda pkg: pkg in installable, pkgs))

            if subprocess.call(['dpkg', '-s'] + pkgs, shell=True,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE) != 0:
                install = True
        elif self.distro in ['CentOS', 'CentOS Linux', 'Fedora', 'Fedora Linux', 'Fedora Linux Asahi Remix']:
            installed_pkgs = str(subprocess.check_output(['rpm', '-qa'])).replace('\n', '|')
            pkgs = DNF_PKGS
            for pkg in pkgs:
                command = ['dnf', 'install']
                if "|{}".format(pkg) not in installed_pkgs:
                    install = True
                    break
        elif self.distro == 'void':
            installed_pkgs = str(subprocess.check_output(['xbps-query', '-l']))
            pkgs = XBPS_PKGS
            for pkg in pkgs:
                command = ['xbps-install', '-A']
                if "ii {}-".format(pkg) not in installed_pkgs:
                    install = force = True
                    break

        if not install:
            return False

        def run_as_root(command, force=False):
            if os.geteuid() != 0:
                command.insert(0, 'sudo')
            if force:
                command.append('-y')
            return subprocess.call(command)

        print("Installing missing dependencies...")
        if run_as_root(command + pkgs, force) != 0:
            raise EnvironmentError("Installation of dependencies failed.")
        return True

    def gstreamer_root(self, cross_compilation_target: Optional[str]) -> Optional[str]:
        return None

    def _platform_bootstrap_gstreamer(self, _force: bool) -> bool:
        raise EnvironmentError(
            "Bootstrapping GStreamer on Linux is not supported. "
            + "Please install it using your distribution package manager.")
