/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var expect = undefined;
var actual = (function foo() { "bogus"; })();

assert.sameValue(expect, actual, "ok");
