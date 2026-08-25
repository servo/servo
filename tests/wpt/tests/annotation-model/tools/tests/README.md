Tree to Test the Tests
======================

This folder is meant to contain a collection of .jsonld files that mirror the
structure of the top level folders and subfolders with .test files.  A script
(@@@TODO@@@) will walk this tree, taking each folder and running it through the
corresponding .test file to ensure that the test is working as expected.  An
argument to that script will report on any .jsonld files that do not have a
corresponding .test file, and vice versa.

