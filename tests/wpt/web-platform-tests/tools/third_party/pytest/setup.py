import os
import sys
import setuptools
import pkg_resources
from setuptools import setup

classifiers = [
    "Development Status :: 6 - Mature",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Operating System :: POSIX",
    "Operating System :: Microsoft :: Windows",
    "Operating System :: MacOS :: MacOS X",
    "Topic :: Software Development :: Testing",
    "Topic :: Software Development :: Libraries",
    "Topic :: Utilities",
] + [
    ("Programming Language :: Python :: %s" % x)
    for x in "2 2.7 3 3.4 3.5 3.6 3.7".split()
]

with open("README.rst") as fd:
    long_description = fd.read()


def get_environment_marker_support_level():
    """
    Tests how well setuptools supports PEP-426 environment marker.

    The first known release to support it is 0.7 (and the earliest on PyPI seems to be 0.7.2
    so we're using that), see: https://setuptools.readthedocs.io/en/latest/history.html#id350

    The support is later enhanced to allow direct conditional inclusions inside install_requires,
    which is now recommended by setuptools. It first appeared in 36.2.0, went broken with 36.2.1, and
    again worked since 36.2.2, so we're using that. See:
    https://setuptools.readthedocs.io/en/latest/history.html#v36-2-2
    https://github.com/pypa/setuptools/issues/1099

    References:

    * https://wheel.readthedocs.io/en/latest/index.html#defining-conditional-dependencies
    * https://www.python.org/dev/peps/pep-0426/#environment-markers
    * https://setuptools.readthedocs.io/en/latest/setuptools.html#declaring-platform-specific-dependencies
    """
    try:
        version = pkg_resources.parse_version(setuptools.__version__)
        if version >= pkg_resources.parse_version("36.2.2"):
            return 2
        if version >= pkg_resources.parse_version("0.7.2"):
            return 1
    except Exception as exc:
        sys.stderr.write("Could not test setuptool's version: %s\n" % exc)

    # as of testing on 2018-05-26 fedora was on version 37* and debian was on version 33+
    # we should consider erroring on those
    return 0


def main():
    extras_require = {}
    install_requires = [
        "py>=1.5.0",
        "six>=1.10.0",
        "setuptools",
        "attrs>=17.4.0",
        "more-itertools>=4.0.0",
        "atomicwrites>=1.0",
    ]
    # if _PYTEST_SETUP_SKIP_PLUGGY_DEP is set, skip installing pluggy;
    # used by tox.ini to test with pluggy master
    if "_PYTEST_SETUP_SKIP_PLUGGY_DEP" not in os.environ:
        install_requires.append("pluggy>=0.5,<0.7")
    environment_marker_support_level = get_environment_marker_support_level()
    if environment_marker_support_level >= 2:
        install_requires.append('funcsigs;python_version<"3.0"')
        install_requires.append('colorama;sys_platform=="win32"')
    elif environment_marker_support_level == 1:
        extras_require[':python_version<"3.0"'] = ["funcsigs"]
        extras_require[':sys_platform=="win32"'] = ["colorama"]
    else:
        if sys.platform == "win32":
            install_requires.append("colorama")
        if sys.version_info < (3, 0):
            install_requires.append("funcsigs")

    setup(
        name="pytest",
        description="pytest: simple powerful testing with Python",
        long_description=long_description,
        use_scm_version={"write_to": "src/_pytest/_version.py"},
        url="http://pytest.org",
        project_urls={
            "Source": "https://github.com/pytest-dev/pytest",
            "Tracker": "https://github.com/pytest-dev/pytest/issues",
        },
        license="MIT license",
        platforms=["unix", "linux", "osx", "cygwin", "win32"],
        author=(
            "Holger Krekel, Bruno Oliveira, Ronny Pfannschmidt, "
            "Floris Bruynooghe, Brianna Laugher, Florian Bruhin and others"
        ),
        entry_points={"console_scripts": ["pytest=pytest:main", "py.test=pytest:main"]},
        classifiers=classifiers,
        keywords="test unittest",
        # the following should be enabled for release
        setup_requires=["setuptools-scm"],
        package_dir={"": "src"},
        python_requires=">=2.7, !=3.0.*, !=3.1.*, !=3.2.*, !=3.3.*",
        install_requires=install_requires,
        extras_require=extras_require,
        packages=[
            "_pytest",
            "_pytest.assertion",
            "_pytest._code",
            "_pytest.mark",
            "_pytest.config",
        ],
        py_modules=["pytest"],
        zip_safe=False,
    )


if __name__ == "__main__":
    main()
