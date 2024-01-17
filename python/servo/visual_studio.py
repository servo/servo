# Copyright 2024 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import dataclasses
import json
import os
import subprocess
import sys
from typing import List

import servo.platform


@dataclasses.dataclass(kw_only=True)
class VisualStudioInstallation:
    version_number: str
    installation_path: str
    vc_install_path: str


def find_highest_msvc_version_ext():
    """Try to find the MSVC installation with the `vswhere.exe` tool. The results
    are sorted with newer versions first."""
    def vswhere(args):
        program_files = (os.environ.get('PROGRAMFILES(X86)')
                         or os.environ.get('PROGRAMFILES'))
        if not program_files:
            return []
        vswhere = os.path.join(program_files, 'Microsoft Visual Studio', 'Installer', 'vswhere.exe')
        if not os.path.exists(vswhere):
            return []
        output = subprocess.check_output([vswhere, '-format', 'json'] + args).decode(errors='ignore')
        return json.loads(output)

    for install in vswhere(['-products', '*',
                            '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
                            '-requires', 'Microsoft.VisualStudio.Component.Windows10SDK']):
        version = install['installationVersion'].split('.')[0] + '.0'
        yield (install['installationPath'], version)


def find_highest_msvc_version():
    prog_files = os.environ.get("ProgramFiles(x86)")

    # TODO(mrobinson): Add support for Visual Studio 2022.
    vs_versions = {
        "2019": "16.0",
    }

    for (version, version_number) in vs_versions.items():
        for edition in ["Enterprise", "Professional", "Community", "BuildTools"]:
            vsinstalldir = os.path.join(prog_files, "Microsoft Visual Studio", version, edition)
            if os.path.exists(vsinstalldir):
                return (vsinstalldir, version_number)

    versions = sorted(find_highest_msvc_version_ext(), key=lambda tup: float(tup[1]))
    if not versions:
        print("Can't find a Visual Studio installation. "
              "Please set the VSINSTALLDIR and VisualStudioVersion environment variables")
        sys.exit(1)
    return versions[0]


def find_msvc() -> VisualStudioInstallation:
    vsinstalldir = os.environ.get('VSINSTALLDIR')
    version_number = os.environ.get('VisualStudioVersion')
    if not vsinstalldir or not version_number:
        (vsinstalldir, version_number) = find_highest_msvc_version()

    vc_install_path = os.environ.get("VCINSTALLDIR", os.path.join(vsinstalldir, "VC"))
    if not os.path.exists(vc_install_path):
        print(f"Can't find Visual C++ {version_number} installation at {vc_install_path}")
        sys.exit(1)

    return VisualStudioInstallation(
        version_number=version_number,
        installation_path=vsinstalldir,
        vc_install_path=vc_install_path,
    )


def find_windows_sdk_installation_path(vs_platform: str) -> str:
    """Try to find the Windows SDK installation path using the Windows registry.
    Raises an Exception if the path cannot be found in the registry."""

    # This module must be imported here, because other platforms also
    # load this file and the module is platform-specific.
    import winreg

    # This is based on the advice from
    # https://stackoverflow.com/questions/35119223/how-to-programmatically-detect-and-locate-the-windows-10-sdk
    key_path = r'SOFTWARE\Wow6432Node\Microsoft\Microsoft SDKs\Windows\v10.0'
    try:
        with winreg.OpenKeyEx(winreg.HKEY_LOCAL_MACHINE, key_path) as key:
            path = str(winreg.QueryValueEx(key, "InstallationFolder")[0])
            return os.path.join(path, "Redist", "ucrt", "DLLs", vs_platform)
    except FileNotFoundError:
        raise Exception(f"Couldn't find Windows SDK installation path in registry at path ({key_path})")


def find_msvc_redist_dirs(target: str) -> List[str]:
    assert 'windows' in servo.platform.host_triple()

    installation = find_msvc()
    msvc_redist_dir = None
    vs_platforms = {
        "x86_64": "x64",
        "i686": "x86",
        "aarch64": "arm64",
    }
    target_arch = target.split('-')[0]
    vs_platform = vs_platforms[target_arch]

    redist_dir = os.path.join(installation.vc_install_path, "Redist", "MSVC")
    if not os.path.isdir(redist_dir):
        raise Exception(f"Couldn't locate MSVC redistributable directory {redist_dir}")

    for p in os.listdir(redist_dir)[::-1]:
        redist_path = os.path.join(redist_dir, p)
        for v in ["VC141", "VC142", "VC150", "VC160"]:
            # there are two possible paths
            # `x64\Microsoft.VC*.CRT` or `onecore\x64\Microsoft.VC*.CRT`
            redist1 = os.path.join(redist_path, vs_platform, "Microsoft.{}.CRT".format(v))
            redist2 = os.path.join(redist_path, "onecore", vs_platform, "Microsoft.{}.CRT".format(v))
            if os.path.isdir(redist1):
                msvc_redist_dir = redist1
                break
            elif os.path.isdir(redist2):
                msvc_redist_dir = redist2
                break
        if msvc_redist_dir:
            break

    if not msvc_redist_dir:
        print("Couldn't locate MSVC redistributable directory")
        sys.exit(1)

    redist_dirs = [
        msvc_redist_dir,
        find_windows_sdk_installation_path(vs_platform)
    ]

    return redist_dirs
