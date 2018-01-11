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


vendor-imports/ Directory
-------------------------

vendor-imports/ is a legacy directory where third parties historically imported
their tests that originate and are maintained in an external repo. Files in
this directory should never be modified in this repo, but should go through the
vendor's process to be imported here.


Importing Old Branches
----------------------

Given an old branch in git based against the old csswg-test
repository, it can be moved over to the merged repo in one of two
ways:

 * (Recommended:) Rebasing on top of web-platform-tests: with the old
   branch checked out, run `git rebase -Xsubtree=css/ origin/master`
   (or similar, depending on the name of the upstream remote).

 * Merging to web-platform-tests: with web-platform-tests' master
   branch checked out, run `git merge -Xsubtree=css/ my_shiny_branch`
   (or similar, depending on the name of your branch).

If you have a branch/bookmark in Mercurial, the process is more
complicated:

 1. From the Mercurial repo, run `hg export --git -r 'outgoing()' >
    foo.patch`. This will export all the changeset shown in `hg log -r
    'outgoing()'`; it's recommended you check this is the right set of
    changesets before continuing!

 2. Move to the git repo, and create a new branch based on
    web-platform-tests' master; e.g., `git checkout -b hg-import
    origin/master` (or similar, depending on the name of the upstream
    remote).

 3. Download [hg-patch-to-git-patch][] and run `python2
    hg-patch-to-git-patch < foo.patch > bar.patch` (where `foo.patch`
    is the path to the `foo.patch` you exported above).

 4. Run `git am --directory=css/ < bar.patch`.


[harness]: https://test.csswg.org/harness/
[spec-link]: http://web-platform-tests.org/writing-tests/css-metadata.html#specification-links
[hg-patch-to-git-patch]: https://raw.githubusercontent.com/mozilla/moz-git-tools/master/hg-patch-to-git-patch
