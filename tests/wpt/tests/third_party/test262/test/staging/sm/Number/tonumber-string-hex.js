/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Various tests of ToNumber(string), particularly +"0x" being NaN
info: bugzilla.mozilla.org/show_bug.cgi?id=872853
esid: pending
---*/

assert.sameValue(+"0x", NaN);
assert.sameValue(+"\t0x", NaN);
assert.sameValue(+"0x\n", NaN);
assert.sameValue(+"\n0x\t", NaN);
assert.sameValue(+"0x0", 0);
assert.sameValue(+"0xa", 10);
assert.sameValue(+"0xff", 255);
assert.sameValue(+"-0x", NaN);
assert.sameValue(+"-0xa", NaN);
assert.sameValue(+"-0xff", NaN);
assert.sameValue(+"0xInfinity", NaN);
assert.sameValue(+"+Infinity", Infinity);
assert.sameValue(+"-Infinity", -Infinity);
assert.sameValue(+"\t+Infinity", Infinity);
assert.sameValue(+"-Infinity\n", -Infinity);
assert.sameValue(+"+ Infinity", NaN);
assert.sameValue(+"- Infinity", NaN);
