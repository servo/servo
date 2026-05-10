/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Treat non-array, non-callable replacers as if none had been specified
info: bugzilla.mozilla.org/show_bug.cgi?id=653782
esid: pending
---*/

var obj = { p: 2 };
var str = '{"p":2}';

assert.sameValue(JSON.stringify(obj), str);
assert.sameValue(JSON.stringify(obj, ["p"]), str);
assert.sameValue(JSON.stringify(obj, null), str);
assert.sameValue(JSON.stringify(obj, undefined), str);
assert.sameValue(JSON.stringify(obj, 2), str);
assert.sameValue(JSON.stringify(obj, Math.PI), str);
assert.sameValue(JSON.stringify(obj, NaN), str);
assert.sameValue(JSON.stringify(obj, true), str);
assert.sameValue(JSON.stringify(obj, false), str);
assert.sameValue(JSON.stringify(obj, Infinity), str);
assert.sameValue(JSON.stringify(obj, "foopy"), str);
assert.sameValue(JSON.stringify(obj, {}), str);
assert.sameValue(JSON.stringify(obj, /abcd/), str);
assert.sameValue(JSON.stringify(obj, new Boolean(true)), str);
assert.sameValue(JSON.stringify(obj, new Number(42)), str);
assert.sameValue(JSON.stringify(obj, new String("aequorin")), str);
