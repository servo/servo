# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import

from setuptools import setup


VERSION = '1.0.0'

README = open('README.rst').read()

setup(
    name='mach',
    description='Generic command line command dispatching framework.',
    long_description=README,
    license='MPL 2.0',
    author='Gregory Szorc',
    author_email='gregory.szorc@gmail.com',
    url='https://developer.mozilla.org/en-US/docs/Developer_Guide/mach',
    packages=['mach', 'mach.mixin'],
    version=VERSION,
    classifiers=[
        'Environment :: Console',
        'Development Status :: 5 - Production/Stable',
        'License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)',
        'Natural Language :: English',
        'Programming Language :: Python :: 2.7',
        'Programming Language :: Python :: 3.5',
    ],
    install_requires=[
        'blessings',
        'mozfile',
        'mozprocess',
        'six',
    ],
    tests_require=['mock'],
)
