# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from setuptools import setup, find_packages

PACKAGE_NAME = 'mozlog'
PACKAGE_VERSION = '2.10'

setup(name=PACKAGE_NAME,
      version=PACKAGE_VERSION,
      description="Robust log handling specialized for logging in the Mozilla universe",
      long_description="see http://mozbase.readthedocs.org/",
      author='Mozilla Automation and Testing Team',
      author_email='tools@lists.mozilla.org',
      url='https://wiki.mozilla.org/Auto-tools/Projects/Mozbase',
      license='MPL 1.1/GPL 2.0/LGPL 2.1',
      packages=find_packages(),
      zip_safe=False,
      install_requires=["blessings>=1.3"],
      tests_require=['mozfile'],
      platforms =['Any'],
      classifiers=['Development Status :: 4 - Beta',
                   'Environment :: Console',
                   'Intended Audience :: Developers',
                   'License :: OSI Approved :: Mozilla Public License 1.1 (MPL 1.1)',
                   'Operating System :: OS Independent',
                   'Topic :: Software Development :: Libraries :: Python Modules',
                  ],
      package_data={"mozlog.structured": ["formatters/html/main.js",
                                          "formatters/html/style.css"]},
      entry_points={
          "console_scripts": [
              "structlog = mozlog.structured.scripts:main"
          ]}
     )
