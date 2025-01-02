#!/usr/bin/env python
# This file is dual licensed under the terms of the Apache License, Version
# 2.0, and the BSD License. See the LICENSE file in the root of this repository
# for complete details.

import os
import re

# While I generally consider it an antipattern to try and support both
# setuptools and distutils with a single setup.py, in this specific instance
# where packaging is a dependency of setuptools, it can create a circular
# dependency when projects attempt to unbundle stuff from setuptools and pip.
# Though we don't really support that, it makes things easier if we do this and
# should hopefully cause less issues for end users.
try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup


base_dir = os.path.dirname(__file__)

about = {}
with open(os.path.join(base_dir, "packaging", "__about__.py")) as f:
    exec(f.read(), about)

with open(os.path.join(base_dir, "README.rst")) as f:
    long_description = f.read()

with open(os.path.join(base_dir, "CHANGELOG.rst")) as f:
    # Remove :issue:`ddd` tags that breaks the description rendering
    changelog = re.sub(
        r":issue:`(\d+)`",
        r"`#\1 <https://github.com/pypa/packaging/issues/\1>`__",
        f.read(),
    )
    long_description = "\n".join([long_description, changelog])


setup(
    name=about["__title__"],
    version=about["__version__"],
    description=about["__summary__"],
    long_description=long_description,
    long_description_content_type="text/x-rst",
    license=about["__license__"],
    url=about["__uri__"],
    author=about["__author__"],
    author_email=about["__email__"],
    python_requires=">=3.6",
    install_requires=["pyparsing>=2.0.2,!=3.0.5"],  # 2.0.2 + needed to avoid issue #91
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "License :: OSI Approved :: BSD License",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: Python :: Implementation :: PyPy",
    ],
    packages=["packaging"],
    package_data={"packaging": ["py.typed"]},
)
