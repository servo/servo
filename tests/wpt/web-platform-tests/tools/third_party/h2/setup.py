#!/usr/bin/env python
# -*- coding: utf-8 -*-
import codecs
import os
import re
import sys

try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup

# Get the version
version_regex = r'__version__ = ["\']([^"\']*)["\']'
with open('h2/__init__.py', 'r') as f:
    text = f.read()
    match = re.search(version_regex, text)

    if match:
        version = match.group(1)
    else:
        raise RuntimeError("No version number found!")

# Stealing this from Kenneth Reitz
if sys.argv[-1] == 'publish':
    os.system('python setup.py sdist upload')
    sys.exit()

packages = [
    'h2',
]

readme = codecs.open('README.rst', encoding='utf-8').read()
history = codecs.open('HISTORY.rst', encoding='utf-8').read()

setup(
    name='h2',
    version=version,
    description='HTTP/2 State-Machine based protocol implementation',
    long_description=u'\n\n'.join([readme, history]),
    author='Cory Benfield',
    author_email='cory@lukasa.co.uk',
    url='http://hyper.rtfd.org',
    packages=packages,
    package_data={'': ['LICENSE', 'README.rst', 'CONTRIBUTORS.rst', 'HISTORY.rst', 'NOTICES']},
    package_dir={'h2': 'h2'},
    include_package_data=True,
    license='MIT License',
    classifiers=[
        'Development Status :: 5 - Production/Stable',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python',
        'Programming Language :: Python :: 2',
        'Programming Language :: Python :: 2.7',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.3',
        'Programming Language :: Python :: 3.4',
        'Programming Language :: Python :: 3.5',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: Implementation :: CPython',
        'Programming Language :: Python :: Implementation :: PyPy',
    ],
    install_requires=[
        'hyperframe>=5.0, <6',
        'hpack>=2.3, <4',
    ],
    extras_require={
        ':python_version == "2.7" or python_version == "3.3"': ['enum34>=1.1.6, <2'],
    }
)
