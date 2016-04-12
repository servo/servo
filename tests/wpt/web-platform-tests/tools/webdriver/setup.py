# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from setuptools import setup, find_packages

setup(name="webdriver",
      version="1.0",
      description="WebDriver client compatible with "
                  "the W3C browser automation specification.",
      author="Mozilla Engineering Productivity",
      author_email="tools@lists.mozilla.org",
      license="MPL 2.0",
      packages=find_packages(),
      classifiers=["Development Status :: 4 - Beta",
                   "Intended Audience :: Developers",
                   "License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)",
                   "Operating System :: OS Independent"])
