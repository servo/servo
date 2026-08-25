# Hack to prevent stupid error on exit of `python setup.py test`. (See
# http://www.eby-sarna.com/pipermail/peak/2010-May/003357.html.)
try:
    import multiprocessing  # noqa
except ImportError:
    pass
from re import sub

from setuptools import setup, find_packages


def get_long_description():
    # Fix display issues on PyPI caused by RST markup
    readme = open('README.rst').read()

    version_lines = []
    with open('docs/versions.rst') as infile:
        next(infile)
        for line in infile:
            line = line.rstrip().replace('.. automodule:: more_itertools', '')
            version_lines.append(line)
    version_history = '\n'.join(version_lines)
    version_history = sub(r':func:`([a-zA-Z0-9._]+)`', r'\1', version_history)

    ret = readme + '\n\n' + version_history
    return ret


setup(
    name='more-itertools',
    version='4.2.0',
    description='More routines for operating on iterables, beyond itertools',
    long_description=get_long_description(),
    author='Erik Rose',
    author_email='erikrose@grinchcentral.com',
    license='MIT',
    packages=find_packages(exclude=['ez_setup']),
    install_requires=['six>=1.0.0,<2.0.0'],
    test_suite='more_itertools.tests',
    url='https://github.com/erikrose/more-itertools',
    include_package_data=True,
    classifiers=[
        'Development Status :: 5 - Production/Stable',
        'Intended Audience :: Developers',
        'Natural Language :: English',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 2',
        'Programming Language :: Python :: 2.7',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.2',
        'Programming Language :: Python :: 3.3',
        'Programming Language :: Python :: 3.4',
        'Programming Language :: Python :: 3.5',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
        'Topic :: Software Development :: Libraries'],
    keywords=['itertools', 'iterator', 'iteration', 'filter', 'peek',
              'peekable', 'collate', 'chunk', 'chunked'],
)
