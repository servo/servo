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
import tempfile
from typing import Optional, Tuple

import distro
from .. import util
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
    'libgles2-mesa-dev', 'libglib2.0-dev', 'libgstreamer-plugins-bad1.0-dev',
    'libgstreamer-plugins-base1.0-dev', 'libgstreamer1.0-dev',
    'libharfbuzz-dev', 'liblzma-dev', 'libunwind-dev', 'libunwind-dev',
    'libvulkan1', 'libx11-dev', 'libxcb-render0-dev', 'libxcb-shape0-dev',
    'libxcb-xfixes0-dev', 'libxmu-dev', 'libxmu6', 'libegl1-mesa-dev',
    'llvm-dev', 'm4', 'xorg-dev',
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
            'gstreamer1-plugins-bad-free-devel', 'libjpeg-turbo-devel',
            'zlib', 'libjpeg', 'vulkan-loader']

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
             'clang', 'gstreamer1-devel',
             'gst-plugins-base1-devel', 'gst-plugins-bad1-devel', 'vulkan-loader']

GSTREAMER_URL = \
    "https://github.com/servo/servo-build-deps/releases/download/linux/gstreamer-1.16-x86_64-linux-gnu.20190515.tar.gz"
PREPACKAGED_GSTREAMER_ROOT = \
    os.path.join(util.get_target_dir(), "dependencies", "gstreamer")


class Linux(Base):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.is_linux = True
        (self.distro, self.version) = Linux.get_distro_and_version()

    def library_path_variable_name(self):
        return "LD_LIBRARY_PATH"

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
        ]:
            raise NotImplementedError("mach bootstrap does not support "
                                      f"{self.distro}, please file a bug")

        installed_something = self.install_non_gstreamer_dependencies(force)
        installed_something |= self._platform_bootstrap_gstreamer(force)
        return installed_something

    def linker_flag(self) -> str:
        # the rust-lld binary downloaded by rustup
        # doesn't respect NIX_LDFLAGS and also needs
        # other patches to work correctly. Use system
        # version of lld for now. See
        # https://github.com/NixOS/nixpkgs/issues/220717
        if self.distro.lower() == 'nixos':
            return '-C link-arg=-fuse-ld=lld'
        else:
            return '-Zgcc-ld=lld'

    def install_non_gstreamer_dependencies(self, force: bool) -> bool:
        install = False
        pkgs = []
        if self.distro in ['Ubuntu', 'Debian GNU/Linux', 'Raspbian GNU/Linux']:
            command = ['apt-get', 'install']
            pkgs = APT_PKGS
            if subprocess.call(['dpkg', '-s'] + pkgs,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE) != 0:
                install = True
        elif self.distro in ['CentOS', 'CentOS Linux', 'Fedora', 'Fedora Linux']:
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
        if cross_compilation_target:
            return None
        if os.path.exists(PREPACKAGED_GSTREAMER_ROOT):
            return PREPACKAGED_GSTREAMER_ROOT
        # GStreamer might be installed system-wide, but we do not return a root in this
        # case because we don't have to update environment variables.
        return None

    def _platform_bootstrap_gstreamer(self, force: bool) -> bool:
        if not force and self.is_gstreamer_installed(cross_compilation_target=None):
            return False

        with tempfile.TemporaryDirectory() as temp_dir:
            file_name = os.path.join(temp_dir, GSTREAMER_URL.rsplit('/', maxsplit=1)[-1])
            util.download_file("Pre-packaged GStreamer binaries", GSTREAMER_URL, file_name)

            print(f"Installing GStreamer packages to {PREPACKAGED_GSTREAMER_ROOT}...")
            os.makedirs(PREPACKAGED_GSTREAMER_ROOT, exist_ok=True)

            # Extract, but strip one component from the output, because the package includes
            # a toplevel directory called "./gst/" and we'd like to have the same directory
            # structure on all platforms.
            subprocess.check_call(["tar", "xf", file_name, "-C", PREPACKAGED_GSTREAMER_ROOT,
                                   "--strip-components=2"])

            assert self.is_gstreamer_installed(cross_compilation_target=None)
            return True
