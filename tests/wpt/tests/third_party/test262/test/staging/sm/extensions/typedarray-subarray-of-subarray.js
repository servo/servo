/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  new Int8Array([1, 2, 3]).subarray(1).subarray(1)[0] === 3
info: bugzilla.mozilla.org/show_bug.cgi?id=637643
esid: pending
---*/

var ta = new Int8Array([1, 2, 3]);
assert.sameValue(ta.length, 3);
assert.sameValue(ta[0], 1);
assert.sameValue(ta[1], 2);
assert.sameValue(ta[2], 3);

var sa1 = ta.subarray(1);
assert.sameValue(sa1.length, 2);
assert.sameValue(sa1[0], 2);
assert.sameValue(sa1[1], 3);

var sa2 = sa1.subarray(1);
assert.sameValue(sa2.length, 1);
assert.sameValue(sa2[0], 3);
