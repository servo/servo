# ECMAScript tests

This directory contains tests related to the [ECMA-262](https://tc39.es/ecma262/) and [ECMA-402](https://tc39.es/ecma402/) specifications.

Although these specifications are already covered through [Test262](https://github.com/tc39/test262), occasionally it’s useful to have Web Platform Tests coverage for a subset of ECMAScript functionality. Examples include:

- Any functionality that ECMAScript specifies as “implementation-defined” despite Web Compatibility requiring specific semantics. ([example](https://github.com/web-platform-tests/wpt/pull/41760))
- Any ECMAScript functionality that needs to be reported via wpt.fyi (for [example](https://github.com/web-platform-tests/wpt/pull/37928), because it’s included in Interop 2023).
