/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Properly handle explicitly-undefined optional arguments to a bunch of functions
info: bugzilla.mozilla.org/show_bug.cgi?id=373118
esid: pending
---*/

var a;

a = "abc".slice(0, undefined);
assert.sameValue(a, "abc");

a = "abc".substr(0, undefined);
assert.sameValue(a, "abc");

a = "abc".substring(0, undefined);
assert.sameValue(a, "abc");

a = [1, 2, 3].slice(0, undefined);
assert.sameValue(a.join(), '1,2,3');

a = [1, 2, 3].sort(undefined);
assert.sameValue(a.join(), '1,2,3');

assert.sameValue((20).toString(undefined), '20');
