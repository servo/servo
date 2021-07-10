import os
import sys
import setuptools
from distutils.core import setup


if sys.version_info[:2] < (2, 7):
    required = ['ordereddict']
else:
    required = []

# Don't shadow builtin enum package if we are being installed on a
# recent Python.  This causes conflicts since at least 3.6:
# https://bitbucket.org/stoneleaf/enum34/issues/19/enum34-isnt-compatible-with-python-36
if sys.version_info[:2] < (3, 4):
    packages = ['enum']
else:
    packages = []

long_desc = '''\
enum --- support for enumerations
========================================

An enumeration is a set of symbolic names (members) bound to unique, constant
values.  Within an enumeration, the members can be compared by identity, and
the enumeration itself can be iterated over.

    from enum import Enum

    class Fruit(Enum):
        apple = 1
        banana = 2
        orange = 3

    list(Fruit)
    # [<Fruit.apple: 1>, <Fruit.banana: 2>, <Fruit.orange: 3>]

    len(Fruit)
    # 3

    Fruit.banana
    # <Fruit.banana: 2>

    Fruit['banana']
    # <Fruit.banana: 2>

    Fruit(2)
    # <Fruit.banana: 2>

    Fruit.banana is Fruit['banana'] is Fruit(2)
    # True

    Fruit.banana.name
    # 'banana'

    Fruit.banana.value
    # 2

Repository and Issue Tracker at https://bitbucket.org/stoneleaf/enum34.
'''

py2_only = ()
py3_only = ()
make = [
        # 'rst2pdf enum/doc/enum.rst --output=enum/doc/enum.pdf',
        ]


data = dict(
        name='enum34',
        version='1.1.10',
        url='https://bitbucket.org/stoneleaf/enum34',
        packages=packages,
        package_data={
            'enum' : [
                'LICENSE',
                'README',
                'doc/enum.rst',
                'doc/enum.pdf',
                'test.py',
                ]
            },
        license='BSD License',
        description='Python 3.4 Enum backported to 3.3, 3.2, 3.1, 2.7, 2.6, 2.5, and 2.4',
        long_description=long_desc,
        provides=['enum'],
        install_requires=required,
        author='Ethan Furman',
        author_email='ethan@stoneleaf.us',
        classifiers=[
            'Development Status :: 5 - Production/Stable',
            'Intended Audience :: Developers',
            'License :: OSI Approved :: BSD License',
            'Programming Language :: Python',
            'Topic :: Software Development',
            'Programming Language :: Python :: 2.4',
            'Programming Language :: Python :: 2.5',
            'Programming Language :: Python :: 2.6',
            'Programming Language :: Python :: 2.7',
            'Programming Language :: Python :: 3.3',
            ],
        )

if __name__ == '__main__':
    setup(**data)
