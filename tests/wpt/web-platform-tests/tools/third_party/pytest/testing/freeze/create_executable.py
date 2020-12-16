# -*- coding: utf-8 -*-
"""
Generates an executable with pytest runner embedded using PyInstaller.
"""
if __name__ == "__main__":
    import pytest
    import subprocess

    hidden = []
    for x in pytest.freeze_includes():
        hidden.extend(["--hidden-import", x])
    hidden.extend(["--hidden-import", "distutils"])
    args = ["pyinstaller", "--noconfirm"] + hidden + ["runtests_script.py"]
    subprocess.check_call(" ".join(args), shell=True)
