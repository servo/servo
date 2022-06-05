# SPDX-License-Identifier: MIT

import codecs
import os
import platform
import re
import sys

from setuptools import find_packages, setup


###############################################################################

NAME = "attrs"
PACKAGES = find_packages(where="src")
META_PATH = os.path.join("src", "attr", "__init__.py")
KEYWORDS = ["class", "attribute", "boilerplate"]
PROJECT_URLS = {
    "Documentation": "https://www.attrs.org/",
    "Changelog": "https://www.attrs.org/en/stable/changelog.html",
    "Bug Tracker": "https://github.com/python-attrs/attrs/issues",
    "Source Code": "https://github.com/python-attrs/attrs",
    "Funding": "https://github.com/sponsors/hynek",
    "Tidelift": "https://tidelift.com/subscription/pkg/pypi-attrs?"
    "utm_source=pypi-attrs&utm_medium=pypi",
    "Ko-fi": "https://ko-fi.com/the_hynek",
}
CLASSIFIERS = [
    "Development Status :: 5 - Production/Stable",
    "Intended Audience :: Developers",
    "Natural Language :: English",
    "License :: OSI Approved :: MIT License",
    "Operating System :: OS Independent",
    "Programming Language :: Python",
    "Programming Language :: Python :: 2",
    "Programming Language :: Python :: 2.7",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.5",
    "Programming Language :: Python :: 3.6",
    "Programming Language :: Python :: 3.7",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
    "Topic :: Software Development :: Libraries :: Python Modules",
]
INSTALL_REQUIRES = []
EXTRAS_REQUIRE = {
    "docs": ["furo", "sphinx", "zope.interface", "sphinx-notfound-page"],
    "tests_no_zope": [
        # For regression test to ensure cloudpickle compat doesn't break.
        'cloudpickle; python_implementation == "CPython"',
        # 5.0 introduced toml; parallel was broken until 5.0.2
        "coverage[toml]>=5.0.2",
        "hypothesis",
        "pympler",
        "pytest>=4.3.0",  # 4.3.0 dropped last use of `convert`
        "six",
    ],
}
if (
    sys.version_info[:2] >= (3, 6)
    and platform.python_implementation() != "PyPy"
):
    EXTRAS_REQUIRE["tests_no_zope"].extend(["mypy", "pytest-mypy-plugins"])

EXTRAS_REQUIRE["tests"] = EXTRAS_REQUIRE["tests_no_zope"] + ["zope.interface"]
EXTRAS_REQUIRE["dev"] = (
    EXTRAS_REQUIRE["tests"] + EXTRAS_REQUIRE["docs"] + ["pre-commit"]
)

###############################################################################

HERE = os.path.abspath(os.path.dirname(__file__))


def read(*parts):
    """
    Build an absolute path from *parts* and return the contents of the
    resulting file.  Assume UTF-8 encoding.
    """
    with codecs.open(os.path.join(HERE, *parts), "rb", "utf-8") as f:
        return f.read()


META_FILE = read(META_PATH)


def find_meta(meta):
    """
    Extract __*meta*__ from META_FILE.
    """
    meta_match = re.search(
        r"^__{meta}__ = ['\"]([^'\"]*)['\"]".format(meta=meta), META_FILE, re.M
    )
    if meta_match:
        return meta_match.group(1)
    raise RuntimeError("Unable to find __{meta}__ string.".format(meta=meta))


LOGO = """
.. image:: https://www.attrs.org/en/stable/_static/attrs_logo.png
   :alt: attrs logo
   :align: center
"""  # noqa

VERSION = find_meta("version")
URL = find_meta("url")
LONG = (
    LOGO
    + read("README.rst").split(".. teaser-begin")[1]
    + "\n\n"
    + "Release Information\n"
    + "===================\n\n"
    + re.search(
        r"(\d+.\d.\d \(.*?\)\r?\n.*?)\r?\n\r?\n\r?\n----\r?\n\r?\n\r?\n",
        read("CHANGELOG.rst"),
        re.S,
    ).group(1)
    + "\n\n`Full changelog "
    + "<{url}en/stable/changelog.html>`_.\n\n".format(url=URL)
    + read("AUTHORS.rst")
)


if __name__ == "__main__":
    setup(
        name=NAME,
        description=find_meta("description"),
        license=find_meta("license"),
        url=URL,
        project_urls=PROJECT_URLS,
        version=VERSION,
        author=find_meta("author"),
        author_email=find_meta("email"),
        maintainer=find_meta("author"),
        maintainer_email=find_meta("email"),
        keywords=KEYWORDS,
        long_description=LONG,
        long_description_content_type="text/x-rst",
        packages=PACKAGES,
        package_dir={"": "src"},
        python_requires=">=2.7, !=3.0.*, !=3.1.*, !=3.2.*, !=3.3.*, !=3.4.*",
        zip_safe=False,
        classifiers=CLASSIFIERS,
        install_requires=INSTALL_REQUIRES,
        extras_require=EXTRAS_REQUIRE,
        include_package_data=True,
        options={"bdist_wheel": {"universal": "1"}},
    )
