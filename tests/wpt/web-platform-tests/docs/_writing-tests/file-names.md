---
layout: page
title: File Name Flags
order: 2
---

The test filename is significant in determining the type of test it
contains, and enabling specific optional features. This page documents
the various flags available and their meaning.


### Test Type

These flags must be the last element in the filename before the
extension e.g. `foo-manual.html` will indicate a manual test, but
`foo-manual-other.html` will not. Unlike test features, test types
are mutually exclusive.


`-manual`
 : Indicates that a test is a non-automated test.

`-support`
 : Indicates that a file is not a test but a support file.  Not
   required for files in a directory called `resources`, `tools` or
   `support`.

`-visual`
 : Indicates that a file is a visual test.


### Test Features

These flags are preceded by a `.` in the filename, and must
themselves precede any test type flag, but are otherwise unordered.


`.https`
 : Indicates that a test is loaded over HTTPS.

`.sub`
 : Indicates that a test uses the [server-side substitution][]
   feature.

`.window`
 : (js files only) Indicates that the file generates a test in which
    it is run in a Window environment.

`.worker`
 : (js files only) Indicates that the file generates a test in which
    it is run in a dedicated worker environment.

`.any`
 : (js files only) Indicates that the file generates tests in which it
    is [run in multiple scopes][multi-global-tests].

`.tentative`
 : Indicates that a test makes assertions not yet required by any specification,
   or in contradiction to some specification. This is useful when implementation
   experience is needed to inform the specification. It should be apparent in
   context why the test is tentative and what needs to be resolved to make it
   non-tentative.


[server-side substitution]: https://wptserve.readthedocs.io/en/latest/pipes.html#sub
[multi-global-tests]: {{ site.baseurl }}{% link _writing-tests/testharness.md %}#multi-global-tests
