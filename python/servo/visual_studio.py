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
from typing import Generator, List, Optional

COMPATIBLE_MSVC_VERSIONS = {
    "2019": "16.0",
    "2022": "17.0",
}
MSVC_REDIST_VERSIONS = ["VC141", "VC142", "VC143", "VC150", "VC160"]

PROGRAM_FILES = os.environ.get("PROGRAMFILES", "C:\\Program Files")
PROGRAM_FILES_X86 = os.environ.get("ProgramFiles(x86)", "C:\\Program Files (x86)")


@dataclasses.dataclass(frozen=True, kw_only=True)
class VisualStudioInstallation:
    version_number: str
    installation_path: str
    vc_install_path: str

    def __lt__(self, other):
        return self.version_number < other.version_number


def find_vswhere():
    for path in [PROGRAM_FILES, PROGRAM_FILES_X86]:
        if not path:
            continue
        vswhere = os.path.join(path, 'Microsoft Visual Studio', 'Installer', 'vswhere.exe')
        if os.path.exists(vswhere):
            return vswhere
    return None


def find_compatible_msvc_with_vswhere() -> Generator[VisualStudioInstallation, None, None]:
    """Try to find the MSVC installation with the `vswhere.exe` tool. The results
    are sorted with newer versions first."""

    vswhere = find_vswhere()
    if not vswhere:
        return

    output = subprocess.check_output([
        vswhere,
        '-format', 'json',
        '-products', '*',
        '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
        '-requires', 'Microsoft.VisualStudio.Component.Windows10SDK'
    ]).decode(errors='ignore')

    for install in json.loads(output):
        installed_version = f"{install['installationVersion'].split('.')[0]}.0"
        if installed_version not in COMPATIBLE_MSVC_VERSIONS.values():
            continue
        installation_path = install['installationPath']
        yield VisualStudioInstallation(
            version_number=installed_version,
            installation_path=installation_path,
            vc_install_path=os.path.join(installation_path, "VC")
        )


def find_compatible_msvc_with_path() -> Generator[VisualStudioInstallation, None, None]:
    for program_files in [PROGRAM_FILES, PROGRAM_FILES_X86]:
        if not program_files:
            continue
        for (version, version_number) in COMPATIBLE_MSVC_VERSIONS.items():
            for edition in ["Enterprise", "Professional", "Community", "BuildTools"]:
                installation_path = os.path.join(program_files, "Microsoft Visual Studio", version, edition)
                if os.path.exists(installation_path):
                    yield VisualStudioInstallation(
                        version_number=version_number,
                        installation_path=installation_path,
                        vc_install_path=os.path.join(installation_path, "VC")
                    )


def find_compatible_msvc_with_environment_variables() -> Optional[VisualStudioInstallation]:
    installation_path = os.environ.get('VSINSTALLDIR')
    version_number = os.environ.get('VisualStudioVersion')
    if not installation_path or not version_number:
        return None
    vc_install_path = os.environ.get("VCINSTALLDIR", os.path.join(installation_path, "VC"))
    if not os.path.exists(installation_path) or not os.path.exists(vc_install_path):
        return None
    return VisualStudioInstallation(
        version_number=version_number,
        installation_path=installation_path,
        vc_install_path=vc_install_path,
    )


def find_msvc_installations() -> List[VisualStudioInstallation]:
    # First try to find Visual Studio via `vswhere.exe` and in well-known paths.
    installations = list(find_compatible_msvc_with_vswhere())
    installations.extend(find_compatible_msvc_with_path())
    if installations:
        return sorted(set(installations), reverse=True)

    # Fall back to using the environment variables, which could theoretically
    # point to a version of Visual Studio that is unsupported.
    installation = find_compatible_msvc_with_environment_variables()
    if installation:
        return [installation]

    raise Exception("Can't find a Visual Studio installation. "
                    "Please set the VSINSTALLDIR and VisualStudioVersion environment variables")


def find_msvc_redist_dirs(vs_platform: str) -> Generator[str, None, None]:
    installations = sorted(set(list(find_msvc_installations())), reverse=True)

    tried = []
    for installation in installations:
        redist_dir = os.path.join(installation.vc_install_path, "Redist", "MSVC")
        if not os.path.isdir(redist_dir):
            tried.append(redist_dir)
            continue

        for subdirectory in os.listdir(redist_dir)[::-1]:
            redist_path = os.path.join(redist_dir, subdirectory)
            for redist_version in MSVC_REDIST_VERSIONS:
                # there are two possible paths
                # `x64\Microsoft.VC*.CRT` or `onecore\x64\Microsoft.VC*.CRT`
                path1 = os.path.join(vs_platform, "Microsoft.{}.CRT".format(redist_version))
                path2 = os.path.join("onecore", vs_platform, "Microsoft.{}.CRT".format(redist_version))
                for path in [path1, path2]:
                    path = os.path.join(redist_path, path)
                    if os.path.isdir(path):
                        yield path
                    else:
                        tried.append(path)

    print("Couldn't locate MSVC redistributable directory. Tried:", file=sys.stderr)
    for path in tried:
        print(f"  * {path}", file=sys.stderr)
    raise Exception("Can't find a MSVC redistributatable directory.")


def find_windows_sdk_installation_path() -> str:
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
            return str(winreg.QueryValueEx(key, "InstallationFolder")[0])
    except FileNotFoundError:
        raise Exception(f"Couldn't find Windows SDK installation path in registry at path ({key_path})")
