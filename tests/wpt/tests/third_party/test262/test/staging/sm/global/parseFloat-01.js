/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  parseFloat(string)
info: bugzilla.mozilla.org/show_bug.cgi?id=613492
esid: pending
---*/

assert.sameValue(parseFloat("Infinity"), Infinity);
assert.sameValue(parseFloat("+Infinity"), Infinity);
assert.sameValue(parseFloat("-Infinity"), -Infinity);

assert.sameValue(parseFloat("inf"), NaN);
assert.sameValue(parseFloat("Inf"), NaN);
assert.sameValue(parseFloat("infinity"), NaN);

assert.sameValue(parseFloat("nan"), NaN);
assert.sameValue(parseFloat("NaN"), NaN);
