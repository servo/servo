# File Name Flags

The test filename is significant in determining the type of test it
contains, and enabling specific optional features. This page documents
the various flags available and their meaning.

In some cases flags can also be set via a directory name, such that any file
that is a (recursive) descendent of the directory inherits the flag value.
These are individually documented for each flag that supports it.


### Test Type

These flags are preceded by a `-` and followed by a `.`, and must be the
last such element in the filename, e.g. `foo-manual.html` will indicate
a manual test, but `foo-manual-other.html` will not. Unlike test features,
test types are mutually exclusive.


`-manual`
 : Indicates that a test is a non-automated test.

`-visual`
 : Indicates that a file is a visual test.


### Test Features

These flags are preceded and followed by a `.` in the filename, and must themselves
go after any test type flag, but are otherwise unordered.


`.https`
 : Indicates that a test is loaded over HTTPS.

 `.h2`
 : Indicates that a test is loaded over HTTP/2.

 `.www`
 : Indicates that a test is run on the `www` subdomain.

`.sub`
 : Indicates that a test uses the [server-side substitution](server-pipes.html#sub)
   feature.

`.window`
 : (js files only) Indicates that the file generates a test in which
    it is run in a Window environment.

`.worker`
 : (js files only) Indicates that the file generates a test in which
    it is run in a dedicated worker environment.

`.any`
 : (js files only) Indicates that the file generates tests in which it
    is [run in multiple scopes](testharness).

`.optional`
 : Indicates that a test makes assertions about optional behavior in a
   specification, typically marked by the [RFC 2119] "MAY" or "OPTIONAL"
   keywords. This flag should not be used for "SHOULD"; such requirements
   can be tested with regular tests, like "MUST".

`.tentative`
 : Indicates that a test makes assertions not yet required by any specification,
   or in contradiction to some specification. This is useful when implementation
   experience is needed to inform the specification. It should be apparent in
   context why the test is tentative and what needs to be resolved to make it
   non-tentative.

   This flag can be enabled for an entire directory (and all its descendents),
   by naming the directory 'tentative'. For example, every test underneath
   'foo/tentative/' will be considered tentative.

It's preferable that `.window`, `.worker`, and `.any` are immediately followed
by their final `.js` extension.

[RFC 2119]: https://tools.ietf.org/html/rfc2119
