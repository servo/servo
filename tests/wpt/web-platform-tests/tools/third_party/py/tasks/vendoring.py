from __future__ import absolute_import, print_function
import os.path
import shutil
import subprocess
import sys

VENDOR_TARGET = "py/_vendored_packages"
GOOD_FILES = ('README.md', '__init__.py')


def remove_libs():
    print("removing vendored libs")
    for filename in os.listdir(VENDOR_TARGET):
        if filename not in GOOD_FILES:
            path = os.path.join(VENDOR_TARGET, filename)
            print(" ", path)
            if os.path.isfile(path):
                os.remove(path)
            else:
                shutil.rmtree(path)


def update_libs():
    print("installing libs")
    subprocess.check_call((
        sys.executable, '-m', 'pip', 'install',
        '--target', VENDOR_TARGET, 'apipkg', 'iniconfig',
    ))
    subprocess.check_call(('git', 'add', VENDOR_TARGET))
    print("Please commit to finish the update after running the tests:")
    print()
    print('    git commit -am "Updated vendored libs"')


def main():
    remove_libs()
    update_libs()


if __name__ == '__main__':
    exit(main())
