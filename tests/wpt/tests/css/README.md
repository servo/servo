Introduction
------------

This directory contains testsuites for CSS WG specifications, including ones
that do not strictly speaking define CSS features, e.g.,
[Geometry Interfaces](https://drafts.fxtf.org/geometry/).

The directories should be named like the specification's shortname, but without
any level suffix.

As the test harness relies on the largely undocumented old CSS build system,
this directory has a number of test requirements specific to it:

 * support files for a given test must live in an adjacent `support` directory;

 * tests must have a [`<link rel=help>`][spec-link] pointing to what they are
   testing;

 * for each spec so linked, test filenames must be unique; and

 * support and reference files must have unique filenames within the entire
   `css` directory.


[spec-link]: https://web-platform-tests.org/writing-tests/css-metadata.html#specification-links
