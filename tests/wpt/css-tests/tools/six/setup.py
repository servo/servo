from __future__ import with_statement

try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup

import six

six_classifiers = [
    "Programming Language :: Python :: 2",
    "Programming Language :: Python :: 3",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Topic :: Software Development :: Libraries",
    "Topic :: Utilities",
]

with open("README", "r") as fp:
    six_long_description = fp.read()

setup(name="six",
      version=six.__version__,
      author="Benjamin Peterson",
      author_email="benjamin@python.org",
      url="http://pypi.python.org/pypi/six/",
      py_modules=["six"],
      description="Python 2 and 3 compatibility utilities",
      long_description=six_long_description,
      license="MIT",
      classifiers=six_classifiers
      )
