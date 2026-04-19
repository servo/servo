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
eval('o.\\u82f1 = 42;');
assert.sameValue(o['\u82f1'], 42);
