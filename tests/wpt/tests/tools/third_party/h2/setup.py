#!/usr/bin/env python3

import os
import re

from setuptools import setup, find_packages

PROJECT_ROOT = os.path.dirname(__file__)

with open(os.path.join(PROJECT_ROOT, 'README.rst')) as file_:
    long_description = file_.read()

version_regex = r'__version__ = ["\']([^"\']*)["\']'
with open(os.path.join(PROJECT_ROOT, 'src/h2/__init__.py')) as file_:
    text = file_.read()
    match = re.search(version_regex, text)
    if match:
        version = match.group(1)
    else:
        raise RuntimeError("No version number found!")

setup(
    name='h2',
    version=version,
    description='HTTP/2 State-Machine based protocol implementation',
    long_description=long_description,
    long_description_content_type='text/x-rst',
    author='Cory Benfield',
    author_email='cory@lukasa.co.uk',
    url='https://github.com/python-hyper/h2',
    packages=find_packages(where="src"),
    package_data={'h2': []},
    package_dir={'': 'src'},
    python_requires='>=3.6.1',
    license='MIT License',
    classifiers=[
        'Development Status :: 5 - Production/Stable',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
        'Programming Language :: Python :: Implementation :: CPython',
        'Programming Language :: Python :: Implementation :: PyPy',
    ],
    install_requires=[
        'hyperframe>=6.0,<7',
        'hpack>=4.0,<5',
    ],
)
