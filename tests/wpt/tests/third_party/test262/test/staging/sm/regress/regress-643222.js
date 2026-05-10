/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
/* This shouldn't trigger an assertion. */
(function () {
    eval("var x=delete(x)")
})();

