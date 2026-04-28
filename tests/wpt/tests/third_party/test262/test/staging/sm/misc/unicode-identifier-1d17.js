/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

var o = {}
eval('o.\\u1d17 = 42;');
assert.sameValue(o['\u1d17'], 42);
