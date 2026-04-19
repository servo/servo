/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - onlyStrict
description: |
  pending
esid: pending
---*/

assert.throws(
    SyntaxError,
    () => eval("(function() { eval(); function eval() {} })")
)
