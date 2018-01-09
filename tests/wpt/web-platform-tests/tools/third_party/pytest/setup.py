import os
import sys
import setuptools
import pkg_resources
from setuptools import setup, Command

classifiers = [
    'Development Status :: 6 - Mature',
    'Intended Audience :: Developers',
    'License :: OSI Approved :: MIT License',
    'Operating System :: POSIX',
    'Operating System :: Microsoft :: Windows',
    'Operating System :: MacOS :: MacOS X',
    'Topic :: Software Development :: Testing',
    'Topic :: Software Development :: Libraries',
    'Topic :: Utilities',
] + [
    ('Programming Language :: Python :: %s' % x)
    for x in '2 2.7 3 3.4 3.5 3.6'.split()
]

with open('README.rst') as fd:
    long_description = fd.read()


def has_environment_marker_support():
    """
    Tests that setuptools has support for PEP-426 environment marker support.

    The first known release to support it is 0.7 (and the earliest on PyPI seems to be 0.7.2
    so we're using that), see: http://pythonhosted.org/setuptools/history.html#id142

    References:

    * https://wheel.readthedocs.io/en/latest/index.html#defining-conditional-dependencies
    * https://www.python.org/dev/peps/pep-0426/#environment-markers
    """
    try:
        return pkg_resources.parse_version(setuptools.__version__) >= pkg_resources.parse_version('0.7.2')
    except Exception as exc:
        sys.stderr.write("Could not test setuptool's version: %s\n" % exc)
        return False


def main():
    extras_require = {}
    install_requires = [
        'py>=1.5.0',
        'six>=1.10.0',
        'setuptools',
        'attrs>=17.2.0',
    ]
    # if _PYTEST_SETUP_SKIP_PLUGGY_DEP is set, skip installing pluggy;
    # used by tox.ini to test with pluggy master
    if '_PYTEST_SETUP_SKIP_PLUGGY_DEP' not in os.environ:
        install_requires.append('pluggy>=0.5,<0.7')
    if has_environment_marker_support():
        extras_require[':python_version<"3.0"'] = ['funcsigs']
        extras_require[':sys_platform=="win32"'] = ['colorama']
    else:
        if sys.platform == 'win32':
            install_requires.append('colorama')
        if sys.version_info < (3, 0):
            install_requires.append('funcsigs')

    setup(
        name='pytest',
        description='pytest: simple powerful testing with Python',
        long_description=long_description,
        use_scm_version={
            'write_to': '_pytest/_version.py',
        },
        url='http://pytest.org',
        license='MIT license',
        platforms=['unix', 'linux', 'osx', 'cygwin', 'win32'],
        author=(
            'Holger Krekel, Bruno Oliveira, Ronny Pfannschmidt, '
            'Floris Bruynooghe, Brianna Laugher, Florian Bruhin and others'),
        entry_points={'console_scripts': [
            'pytest=pytest:main', 'py.test=pytest:main']},
        classifiers=classifiers,
        keywords="test unittest",
        cmdclass={'test': PyTest},
        # the following should be enabled for release
        setup_requires=['setuptools-scm'],
        python_requires='>=2.7, !=3.0.*, !=3.1.*, !=3.2.*, !=3.3.*',
        install_requires=install_requires,
        extras_require=extras_require,
        packages=['_pytest', '_pytest.assertion', '_pytest._code'],
        py_modules=['pytest'],
        zip_safe=False,
    )


class PyTest(Command):
    user_options = []

    def initialize_options(self):
        pass

    def finalize_options(self):
        pass

    def run(self):
        import subprocess
        PPATH = [x for x in os.environ.get('PYTHONPATH', '').split(':') if x]
        PPATH.insert(0, os.getcwd())
        os.environ['PYTHONPATH'] = ':'.join(PPATH)
        errno = subprocess.call([sys.executable, 'pytest.py', '--ignore=doc'])
        raise SystemExit(errno)


if __name__ == '__main__':
    main()
