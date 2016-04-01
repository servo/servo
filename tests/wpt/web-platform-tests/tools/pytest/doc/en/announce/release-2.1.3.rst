py.test 2.1.3: just some more fixes
===========================================================================

pytest-2.1.3 is a minor backward compatible maintenance release of the
popular py.test testing tool.  It is commonly used for unit, functional-
and integration testing.  See extensive docs with examples here:

     http://pytest.org/

The release contains another fix to the perfected assertions introduced
with the 2.1 series as well as the new possibility to customize reporting
for assertion expressions on a per-directory level.  

If you want to install or upgrade pytest, just type one of::

    pip install -U pytest # or
    easy_install -U pytest

Thanks to the bug reporters and to Ronny Pfannschmidt, Benjamin Peterson
and Floris Bruynooghe who implemented the fixes.

best,
holger krekel

Changes between 2.1.2 and 2.1.3
----------------------------------------

- fix issue79: assertion rewriting failed on some comparisons in boolops,
- correctly handle zero length arguments (a la pytest '')
- fix issue67 / junitxml now contains correct test durations
- fix issue75 / skipping test failure on jython
- fix issue77 / Allow assertrepr_compare hook to apply to a subset of tests
