from setuptools import setup, find_packages

setup(name="webdriver",
      version="1.0",
      description="WebDriver client compatible with "
                  "the W3C browser automation specification.",
      author="Mozilla Engineering Productivity",
      author_email="tools@lists.mozilla.org",
      license="BSD",
      packages=find_packages(),
      classifiers=["Development Status :: 4 - Beta",
                   "Intended Audience :: Developers",
                   "License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)",
                   "Operating System :: OS Independent"])
