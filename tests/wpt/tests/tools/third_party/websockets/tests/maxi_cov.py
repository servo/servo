#!/usr/bin/env python

"""Measure coverage of each module by its test module."""

import glob
import os.path
import subprocess
import sys


UNMAPPED_SRC_FILES = ["websockets/version.py"]
UNMAPPED_TEST_FILES = ["tests/test_exports.py"]


def check_environment():
    """Check that prerequisites for running this script are met."""
    try:
        import websockets  # noqa: F401
    except ImportError:
        print("failed to import websockets; is src on PYTHONPATH?")
        return False
    try:
        import coverage  # noqa: F401
    except ImportError:
        print("failed to locate Coverage.py; is it installed?")
        return False
    return True


def get_mapping(src_dir="src"):
    """Return a dict mapping each source file to its test file."""

    # List source and test files.

    src_files = glob.glob(
        os.path.join(src_dir, "websockets/**/*.py"),
        recursive=True,
    )
    test_files = glob.glob(
        "tests/**/*.py",
        recursive=True,
    )

    src_files = [
        os.path.relpath(src_file, src_dir)
        for src_file in sorted(src_files)
        if "legacy" not in os.path.dirname(src_file)
        if os.path.basename(src_file) != "__init__.py"
        and os.path.basename(src_file) != "__main__.py"
        and os.path.basename(src_file) != "compatibility.py"
    ]
    test_files = [
        test_file
        for test_file in sorted(test_files)
        if "legacy" not in os.path.dirname(test_file)
        and os.path.basename(test_file) != "__init__.py"
        and os.path.basename(test_file).startswith("test_")
    ]

    # Map source files to test files.

    mapping = {}
    unmapped_test_files = []

    for test_file in test_files:
        dir_name, file_name = os.path.split(test_file)
        assert dir_name.startswith("tests")
        assert file_name.startswith("test_")
        src_file = os.path.join(
            "websockets" + dir_name[len("tests") :],
            file_name[len("test_") :],
        )
        if src_file in src_files:
            mapping[src_file] = test_file
        else:
            unmapped_test_files.append(test_file)

    unmapped_src_files = list(set(src_files) - set(mapping))

    # Ensure that all files are mapped.

    assert unmapped_src_files == UNMAPPED_SRC_FILES
    assert unmapped_test_files == UNMAPPED_TEST_FILES

    return mapping


def get_ignored_files(src_dir="src"):
    """Return the list of files to exclude from coverage measurement."""

    return [
        # */websockets matches src/websockets and .tox/**/site-packages/websockets.
        # There are no tests for the __main__ module and for compatibility modules.
        "*/websockets/__main__.py",
        "*/websockets/*/compatibility.py",
        # This approach isn't applicable to the test suite of the legacy
        # implementation, due to the huge test_client_server test module.
        "*/websockets/legacy/*",
        "tests/legacy/*",
    ] + [
        # Exclude test utilities that are shared between several test modules.
        # Also excludes this script.
        test_file
        for test_file in sorted(glob.glob("tests/**/*.py", recursive=True))
        if "legacy" not in os.path.dirname(test_file)
        and os.path.basename(test_file) != "__init__.py"
        and not os.path.basename(test_file).startswith("test_")
    ]


def run_coverage(mapping, src_dir="src"):
    # Initialize a new coverage measurement session. The --source option
    # includes all files in the report, even if they're never imported.
    print("\nInitializing session\n", flush=True)
    subprocess.run(
        [
            sys.executable,
            "-m",
            "coverage",
            "run",
            "--source",
            ",".join([os.path.join(src_dir, "websockets"), "tests"]),
            "--omit",
            ",".join(get_ignored_files(src_dir)),
            "-m",
            "unittest",
        ]
        + UNMAPPED_TEST_FILES,
        check=True,
    )
    # Append coverage of each source module by the corresponding test module.
    for src_file, test_file in mapping.items():
        print(f"\nTesting {src_file} with {test_file}\n", flush=True)
        subprocess.run(
            [
                sys.executable,
                "-m",
                "coverage",
                "run",
                "--append",
                "--include",
                ",".join([os.path.join(src_dir, src_file), test_file]),
                "-m",
                "unittest",
                test_file,
            ],
            check=True,
        )


if __name__ == "__main__":
    if not check_environment():
        sys.exit(1)
    src_dir = sys.argv[1] if len(sys.argv) == 2 else "src"
    mapping = get_mapping(src_dir)
    run_coverage(mapping, src_dir)
