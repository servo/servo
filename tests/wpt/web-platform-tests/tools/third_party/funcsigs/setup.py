#!/usr/bin/env python
from setuptools import setup
import re
import sys

def load_version(filename='funcsigs/version.py'):
    "Parse a __version__ number from a source file"
    with open(filename) as source:
        text = source.read()
        match = re.search(r"^__version__ = ['\"]([^'\"]*)['\"]", text)
        if not match:
            msg = "Unable to find version number in {}".format(filename)
            raise RuntimeError(msg)
        version = match.group(1)
        return version

def load_rst(filename='docs/source/guide_content.rst'):
    "Purge refs directives from restructured text"
    with open(filename) as source:
        text = source.read()
        doc = re.sub(r':\w+:`~?([a-zA-Z._()]+)`', r'*\1*', text)
        return doc

setup(
    name="funcsigs",
    version=load_version(),
    packages=['funcsigs'],
    zip_safe=False,
    author="Aaron Iles",
    author_email="aaron.iles@gmail.com",
    url="http://funcsigs.readthedocs.org",
    description="Python function signatures from PEP362 for Python 2.6, 2.7 and 3.2+",
    long_description=open('README.rst').read(),
    # long_description=load_rst(),
    license="ASL",
    install_requires = [],
    classifiers = [
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: Apache Software License',
        'Operating System :: OS Independent',
        'Programming Language :: Python',
        'Programming Language :: Python :: 2',
        'Programming Language :: Python :: 2.6',
        'Programming Language :: Python :: 2.7',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.2',
        'Programming Language :: Python :: 3.3',
        'Programming Language :: Python :: Implementation :: CPython',
        'Programming Language :: Python :: Implementation :: PyPy',
        'Topic :: Software Development :: Libraries :: Python Modules'
    ],
    tests_require = [] if sys.version_info[0] > 2 else ['unittest2'],
    test_suite = "tests" if sys.version_info[0] > 2 else 'unittest2.collector'
)
