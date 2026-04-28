/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Sorting an array containing only holes and |undefined| should move all |undefined| to the start of the array
info: bugzilla.mozilla.org/show_bug.cgi?id=664528
esid: pending
---*/

var a = [, , , undefined];
a.sort();

assert.sameValue(a.hasOwnProperty(0), true);
assert.sameValue(a[0], undefined);
assert.sameValue(a.hasOwnProperty(1), false);
assert.sameValue(a.hasOwnProperty(2), false);
assert.sameValue(a.hasOwnProperty(3), false);
