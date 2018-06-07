from __future__ import print_function

import ast
import codecs
import sys

from os.path import join, dirname
from setuptools import setup, find_packages, __version__ as setuptools_version
from pkg_resources import parse_version

import pkg_resources

try:
    import _markerlib.markers
except ImportError:
    _markerlib = None


# _markerlib.default_environment() obtains its data from _VARS
# and wraps it in another dict, but _markerlib_evaluate writes
# to the dict while it is iterating the keys, causing an error
# on Python 3 only.
# Replace _markerlib.default_environment to return a custom dict
# that has all the necessary markers, and ignores any writes.

class Python3MarkerDict(dict):

    def __setitem__(self, key, value):
        pass

    def pop(self, i=-1):
        return self[i]


if _markerlib and sys.version_info[0] == 3:
    env = _markerlib.markers._VARS
    for key in list(env.keys()):
        new_key = key.replace('.', '_')
        if new_key != key:
            env[new_key] = env[key]

    _markerlib.markers._VARS = Python3MarkerDict(env)

    def default_environment():
        return _markerlib.markers._VARS

    _markerlib.default_environment = default_environment

# Avoid the very buggy pkg_resources.parser, which doesnt consistently
# recognise the markers needed by this setup.py
# Change this to setuptools 20.10.0 to support all markers.
if pkg_resources:
    if parse_version(setuptools_version) < parse_version('18.5'):
        MarkerEvaluation = pkg_resources.MarkerEvaluation

        del pkg_resources.parser
        pkg_resources.evaluate_marker = MarkerEvaluation._markerlib_evaluate
        MarkerEvaluation.evaluate_marker = MarkerEvaluation._markerlib_evaluate

classifiers = [
    'Development Status :: 5 - Production/Stable',
    'Intended Audience :: Developers',
    'License :: OSI Approved :: MIT License',
    'Operating System :: OS Independent',
    'Programming Language :: Python',
    'Programming Language :: Python :: 2',
    'Programming Language :: Python :: 2.7',
    'Programming Language :: Python :: 3',
    'Programming Language :: Python :: 3.3',
    'Programming Language :: Python :: 3.4',
    'Programming Language :: Python :: 3.5',
    'Programming Language :: Python :: 3.6',
    'Topic :: Software Development :: Libraries :: Python Modules',
    'Topic :: Text Processing :: Markup :: HTML'
]

here = dirname(__file__)
with codecs.open(join(here, 'README.rst'), 'r', 'utf8') as readme_file:
    with codecs.open(join(here, 'CHANGES.rst'), 'r', 'utf8') as changes_file:
        long_description = readme_file.read() + '\n' + changes_file.read()

version = None
with open(join(here, "html5lib", "__init__.py"), "rb") as init_file:
    t = ast.parse(init_file.read(), filename="__init__.py", mode="exec")
    assert isinstance(t, ast.Module)
    assignments = filter(lambda x: isinstance(x, ast.Assign), t.body)
    for a in assignments:
        if (len(a.targets) == 1 and
                isinstance(a.targets[0], ast.Name) and
                a.targets[0].id == "__version__" and
                isinstance(a.value, ast.Str)):
            version = a.value.s

setup(name='html5lib',
      version=version,
      url='https://github.com/html5lib/html5lib-python',
      license="MIT License",
      description='HTML parser based on the WHATWG HTML specification',
      long_description=long_description,
      classifiers=classifiers,
      maintainer='James Graham',
      maintainer_email='james@hoppipolla.co.uk',
      packages=find_packages(exclude=["*.tests", "*.tests.*", "tests.*", "tests"]),
      install_requires=[
          'six>=1.9',
          'webencodings',
      ],
      extras_require={
          # A conditional extra will only install these items when the extra is
          # requested and the condition matches.
          "datrie:platform_python_implementation == 'CPython'": ["datrie"],
          "lxml:platform_python_implementation == 'CPython'": ["lxml"],

          # Standard extras, will be installed when the extra is requested.
          "genshi": ["genshi"],
          "chardet": ["chardet>=2.2"],

          # The all extra combines a standard extra which will be used anytime
          # the all extra is requested, and it extends it with a conditional
          # extra that will be installed whenever the condition matches and the
          # all extra is requested.
          "all": ["genshi", "chardet>=2.2"],
          "all:platform_python_implementation == 'CPython'": ["datrie", "lxml"],
      },
      )
