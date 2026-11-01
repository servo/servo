#!/usr/bin/env python
import re

# While I generally consider it an antipattern to try and support both
# setuptools and distutils with a single setup.py, in this specific instance
# where certifi is a dependency of setuptools, it can create a circular
# dependency when projects attempt to unbundle stuff from setuptools and pip.
# Though we don't really support that, it makes things easier if we do this and
# should hopefully cause less issues for end users.
try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup


version_regex = r'__version__ = ["\']([^"\']*)["\']'
with open("certifi/__init__.py") as f:
    text = f.read()
    match = re.search(version_regex, text)

    if match:
        VERSION = match.group(1)
    else:
        raise RuntimeError("No version number found!")

setup(
    name="certifi",
    version=VERSION,
    description="Python package for providing Mozilla's CA Bundle.",
    long_description=open("README.rst").read(),
    author="Kenneth Reitz",
    author_email="me@kennethreitz.com",
    url="https://github.com/certifi/python-certifi",
    packages=[
        "certifi",
    ],
    package_dir={"certifi": "certifi"},
    package_data={"certifi": ["*.pem", "py.typed"]},
    include_package_data=True,
    zip_safe=False,
    license="MPL-2.0",
    python_requires=">=3.7",
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)",
        "Natural Language :: English",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
        "Programming Language :: Python :: 3.15",
    ],
    project_urls={
        "Source": "https://github.com/certifi/python-certifi",
    },
)
