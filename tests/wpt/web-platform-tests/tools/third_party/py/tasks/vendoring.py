from __future__ import absolute_import, print_function
import py
import invoke

VENDOR_TARGET = py.path.local("py/_vendored_packages")
GOOD_FILES = 'README.md', '__init__.py'

@invoke.task()
def remove_libs(ctx):
    print("removing vendored libs")
    for path in VENDOR_TARGET.listdir():
        if path.basename not in GOOD_FILES:
            print(" ", path)
            path.remove()

@invoke.task(pre=[remove_libs])
def update_libs(ctx):
    print("installing libs")
    ctx.run("pip install -t {target} apipkg iniconfig".format(target=VENDOR_TARGET))
    ctx.run("git add {target}".format(target=VENDOR_TARGET))
    print("Please commit to finish the update after running the tests:")
    print()
    print('    git commit -am "Updated vendored libs"')
