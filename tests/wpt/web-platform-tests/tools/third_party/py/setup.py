import os
import sys

from setuptools import setup, find_packages


def get_version():
    p = os.path.join(os.path.dirname(
                     os.path.abspath(__file__)), "py", "__init__.py")
    with open(p) as f:
        for line in f.readlines():
            if "__version__" in line:
                return line.strip().split("=")[-1].strip(" '")
    raise ValueError("could not read version")


def main():
    setup(
        name='py',
        description='library with cross-python path, ini-parsing, io, code, log facilities',
        long_description=open('README.rst').read(),
        version=get_version(),
        url='http://py.readthedocs.io/',
        license='MIT license',
        platforms=['unix', 'linux', 'osx', 'cygwin', 'win32'],
        python_requires='>=2.7, !=3.0.*, !=3.1.*, !=3.2.*, !=3.3.*',
        author='holger krekel, Ronny Pfannschmidt, Benjamin Peterson and others',
        author_email='pytest-dev@python.org',
        classifiers=['Development Status :: 6 - Mature',
                     'Intended Audience :: Developers',
                     'License :: OSI Approved :: MIT License',
                     'Operating System :: POSIX',
                     'Operating System :: Microsoft :: Windows',
                     'Operating System :: MacOS :: MacOS X',
                     'Topic :: Software Development :: Testing',
                     'Topic :: Software Development :: Libraries',
                     'Topic :: Utilities',
                     'Programming Language :: Python',
                     'Programming Language :: Python :: 2',
                     'Programming Language :: Python :: 2.7',
                     'Programming Language :: Python :: 3',
                     'Programming Language :: Python :: 3.4',
                     'Programming Language :: Python :: 3.5',
                     'Programming Language :: Python :: 3.6',
                     'Programming Language :: Python :: Implementation :: CPython',
                     'Programming Language :: Python :: Implementation :: PyPy',
                    ],
        packages=find_packages(exclude=['tasks', 'testing']),
        zip_safe=False,
    )

if __name__ == '__main__':
    main()
