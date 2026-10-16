#!/usr/bin/env python3
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Windows Hardware GPU and Display Environment Checker for Servo Embedders."""

import platform
import subprocess
import sys


def main():
    print("=" * 60)
    print("Servo Windows Hardware Validation Environment")
    print("=" * 60)

    if sys.platform != "win32":
        print(f"Skipping: OS is {sys.platform}, not Windows.")
        return 0

    print(f"OS Version: {platform.platform()} ({platform.version()})")
    print(f"Architecture: {platform.machine()}")

    # Query GPU via PowerShell / WMI
    try:
        ps_cmd = (
            "Get-CimInstance Win32_VideoController | "
            "Select-Object Name, DriverVersion, AdapterRAM, VideoProcessor | "
            "Format-List"
        )
        res = subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps_cmd],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if res.returncode == 0:
            print("\nDetected GPU Adapters:")
            print(res.stdout.strip())
        else:
            print("Failed to query Win32_VideoController via WMI.")
    except Exception as e:
        print(f"Error querying GPU: {e}")

    # Query Displays
    try:
        ps_cmd = (
            "Get-CimInstance Win32_DesktopMonitor | "
            "Select-Object DeviceID, ScreenHeight, ScreenWidth | "
            "Format-Table"
        )
        res = subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps_cmd],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if res.returncode == 0:
            print("\nDetected Display Monitors:")
            print(res.stdout.strip())
    except Exception as e:
        print(f"Error querying monitors: {e}")

    print("\nEnvironment check completed successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
