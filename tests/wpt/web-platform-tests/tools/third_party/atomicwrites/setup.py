# -*- coding: utf-8 -*-

import ast
import re

from setuptools import find_packages, setup


_version_re = re.compile(r'__version__\s+=\s+(.*)')


with open('atomicwrites/__init__.py', 'rb') as f:
    version = str(ast.literal_eval(_version_re.search(
        f.read().decode('utf-8')).group(1)))

setup(
    name='atomicwrites',
    version=version,
    author='Markus Unterwaditzer',
    author_email='markus@unterwaditzer.net',
    url='https://github.com/untitaker/python-atomicwrites',
    description='Atomic file writes.',
    license='MIT',
    long_description=open('README.rst').read(),
    packages=find_packages(exclude=['tests.*', 'tests']),
    include_package_data=True,
)
