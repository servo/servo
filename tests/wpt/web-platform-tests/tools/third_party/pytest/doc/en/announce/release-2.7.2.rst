pytest-2.7.2: bug fixes
=======================

pytest is a mature Python testing tool with more than a 1100 tests
against itself, passing on many different interpreters and platforms.
This release is supposed to be drop-in compatible to 2.7.1.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

    Bruno Oliveira
    Floris Bruynooghe
    Punyashloka Biswal
    Aron Curzon
    Benjamin Peterson
    Thomas De Schampheleire
    Edison Gustavo Muenz
    Holger Krekel

Happy testing,
The py.test Development Team


2.7.2 (compared to 2.7.1)
-----------------------------

- fix issue767: pytest.raises value attribute does not contain the exception
  instance on Python 2.6. Thanks Eric Siegerman for providing the test
  case and Bruno Oliveira for PR.

- Automatically create directory for junitxml and results log.
  Thanks Aron Curzon.

- fix issue713: JUnit XML reports for doctest failures.
  Thanks Punyashloka Biswal.

- fix issue735: assertion failures on debug versions of Python 3.4+
  Thanks Benjamin Peterson.

- fix issue114: skipif marker reports to internal skipping plugin;
  Thanks Floris Bruynooghe for reporting and Bruno Oliveira for the PR.

- fix issue748: unittest.SkipTest reports to internal pytest unittest plugin.
  Thanks Thomas De Schampheleire for reporting and Bruno Oliveira for the PR.

- fix issue718: failed to create representation of sets containing unsortable
  elements in python 2. Thanks Edison Gustavo Muenz

- fix issue756, fix issue752 (and similar issues): depend on py-1.4.29
  which has a refined algorithm for traceback generation.

