from setuptools import setup

PACKAGE_VERSION = '1.3.0'
deps = []

setup(name='wptserve',
      version=PACKAGE_VERSION,
      description="Python webserver intended for in web browser testing",
      long_description=open("README.md").read(),
      # Get strings from http://pypi.python.org/pypi?%3Aaction=list_classifiers
      classifiers=["Development Status :: 5 - Production/Stable",
                   "License :: OSI Approved :: BSD License",
                   "Topic :: Internet :: WWW/HTTP :: HTTP Servers"],
      keywords='',
      author='James Graham',
      author_email='james@hoppipolla.co.uk',
      url='http://wptserve.readthedocs.org/',
      license='BSD',
      packages=['wptserve'],
      include_package_data=True,
      zip_safe=False,
      install_requires=deps
      )
