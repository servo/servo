# crashtest tests

Crash tests are used to ensure that a document can be loaded without
crashing or experiencing other low-level issues that may be checked by
implementation-specific tooling (e.g. leaks, asserts, or sanitizer
failures).

Crashtests are identified by the string `-crash` in the filename, or
by being in a directory called `crashtests`.

The simplest crashtest is a single HTML file with any content. The
test passes if the load event is reached, and the browser finishes
painting, without terminating.

In some cases crashtests may need to perform work after the initial page load.
In this case the test may specify a `class=test-wait` attribute on the root
element. The test will not complete until that attribute is removed from the
root. At the time when the test would otherwise have ended a `TestRendered`
event is emitted; test authors can use this event to perform modifications that
are guaranteed not to be batched with the initial paint. This matches the
behaviour of [reftests](reftests).

Note that crash tests **do not** need to include `testharness.js` or use any of
the [testharness API](testharness-api.md) (e.g. they do not need to declare a
`test(..)`).
