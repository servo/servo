"""
Installs cx_freeze from source, but first patching
setup.py as described here:

http://stackoverflow.com/questions/25107697/compiling-cx-freeze-under-ubuntu
"""
import glob
import tarfile
import os
import sys
import platform
import py

if __name__ == '__main__':
    if 'ubuntu' not in platform.version().lower():

        print('Not Ubuntu, installing using pip. (platform.version() is %r)' %
              platform.version())
        res = os.system('pip install cx_freeze')
        if res != 0:
            sys.exit(res)
        sys.exit(0)

    rootdir = py.path.local.make_numbered_dir(prefix='cx_freeze')

    res = os.system('pip install --download %s --no-use-wheel '
                    'cx_freeze' % rootdir)
    if res != 0:
        sys.exit(res)

    packages = glob.glob('%s/*.tar.gz' % rootdir)
    assert len(packages) == 1
    tar_filename = packages[0]

    tar_file = tarfile.open(tar_filename)
    try:
        tar_file.extractall(path=str(rootdir))
    finally:
        tar_file.close()

    basename = os.path.basename(tar_filename).replace('.tar.gz', '')
    setup_py_filename = '%s/%s/setup.py' % (rootdir, basename)
    with open(setup_py_filename) as f:
        lines = f.readlines()

    line_to_patch = 'if not vars.get("Py_ENABLE_SHARED", 0):'
    for index, line in enumerate(lines):
        if line_to_patch in line:
            indent = line[:line.index(line_to_patch)]
            lines[index] = indent + 'if True:\n'
            print('Patched line %d' % (index + 1))
            break
    else:
        sys.exit('Could not find line in setup.py to patch!')

    with open(setup_py_filename, 'w') as f:
        f.writelines(lines)

    os.chdir('%s/%s' % (rootdir, basename))
    res = os.system('python setup.py install')
    if res != 0:
        sys.exit(res)

    sys.exit(0)
