/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
assert.throws(SyntaxError, function() { eval('let a = 100_00_;'); });
assert.throws(SyntaxError, () => eval("let b = 10__;"));
assert.throws(SyntaxError, () => eval("let b = 1._2;"));

