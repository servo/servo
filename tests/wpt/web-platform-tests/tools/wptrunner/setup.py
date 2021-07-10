# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function

import glob
import os
import sys
import textwrap

from setuptools import setup, find_packages

here = os.path.dirname(__file__)

PACKAGE_NAME = 'wptrunner'
PACKAGE_VERSION = '1.14'

# Dependencies
with open(os.path.join(here, "requirements.txt")) as f:
    deps = f.read().splitlines()

# Browser-specific requirements
requirements_files = glob.glob("requirements_*.txt")

profile_dest = None
dest_exists = False

setup(name=PACKAGE_NAME,
      version=PACKAGE_VERSION,
      description="Harness for running the W3C web-platform-tests against various products",
      author='Mozilla Automation and Testing Team',
      author_email='tools@lists.mozilla.org',
      license='MPL 2.0',
      packages=find_packages(exclude=["tests", "metadata", "prefs"]),
      entry_points={
          'console_scripts': [
              'wptrunner = wptrunner.wptrunner:main',
              'wptupdate = wptrunner.update:main',
          ]
      },
      zip_safe=False,
      platforms=['Any'],
      classifiers=['Development Status :: 4 - Beta',
                   'Environment :: Console',
                   'Intended Audience :: Developers',
                   'License :: OSI Approved :: BSD License',
                   'Operating System :: OS Independent'],
      package_data={"wptrunner": ["executors/testharness_marionette.js",
                                  "executors/testharness_webdriver.js",
                                  "executors/reftest.js",
                                  "executors/reftest-wait.js",
                                  "testharnessreport.js",
                                  "testharness_runner.html",
                                  "wptrunner.default.ini",
                                  "browsers/sauce_setup/*",
                                  "prefs/*"]},
      include_package_data=True,
      data_files=[("requirements", requirements_files)],
      )

if "install" in sys.argv:
    path = os.path.relpath(os.path.join(sys.prefix, "requirements"), os.curdir)
    print(textwrap.fill("""In order to use with one of the built-in browser
products, you will need to install the extra dependencies. These are provided
as requirements_[name].txt in the %s directory and can be installed using
e.g.""" % path, 80))

    print("""

pip install -r %s/requirements_firefox.txt
""" % path)
