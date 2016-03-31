"""
Sample setup.py script that generates an executable with pytest runner embedded.
"""
if __name__ == '__main__':
    from cx_Freeze import setup, Executable
    import pytest

    setup(
        name="runtests",
        version="0.1",
        description="exemple of how embedding py.test into an executable using cx_freeze",
        executables=[Executable("runtests_script.py")],
        options={"build_exe": {'includes': pytest.freeze_includes()}},
    )

