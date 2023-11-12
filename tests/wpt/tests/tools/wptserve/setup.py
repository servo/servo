from setuptools import setup

PACKAGE_VERSION = '4.0.1'
deps = [
    "h2>=4.1.0",
    "mod_pywebsocket @ https://github.com/GoogleChromeLabs/pywebsocket3/archive/50602a14f1b6da17e0b619833a13addc6ea78bc2.zip#sha256=4dadd116e67af5625606f883e1973178d4121e8a1dc87b096ba2bb43c692f958",  # noqa
]

setup(name='wptserve',
      version=PACKAGE_VERSION,
      description="Python web server intended for in web browser testing",
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
      packages=['wptserve', 'wptserve.sslutils'],
      include_package_data=True,
      zip_safe=False,
      install_requires=deps
      )
