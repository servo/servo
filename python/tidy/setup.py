# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
from setuptools import setup, find_packages


VERSION = '0.3.0'

install_requires = [
    "flake8==3.8.3",
    "toml==0.9.2",
    "colorama==0.3.7",
    "voluptuous==0.11.5",
    "PyYAML==5.4",
]

here = os.path.dirname(os.path.abspath(__file__))
# get documentation from the README and HISTORY
try:
    with open(os.path.join(here, 'README.rst')) as doc:
        readme = doc.read()
except Exception:
    readme = ''

try:
    with open(os.path.join(here, 'HISTORY.rst')) as doc:
        history = doc.read()
except Exception:
    history = ''

long_description = readme + '\n\n' + history

if __name__ == '__main__':
    setup(
        name='servo_tidy',
        version=VERSION,
        description='The servo-tidy is used to check licenses, '
                    'line lengths, whitespace, flake8 on Python files, lock file versions, and more.',
        long_description=long_description,
        keywords='mozilla servo tidy ',
        author='The Servo Project Developers',
        author_email='dev-servo@lists.mozilla.org',
        url='https://github.com/servo/servo',
        packages=find_packages(exclude=['ez_setup', 'examples', 'tests']),
        package_data={},
        install_requires=install_requires,
        zip_safe=False,
        entry_points={
            'console_scripts': [
                'servo-tidy=servo_tidy.tidy:scan',
            ],
        },
    )
