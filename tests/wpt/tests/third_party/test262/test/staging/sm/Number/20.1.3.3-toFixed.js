/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
assert.sameValue(Number.prototype.toFixed.call(-Infinity), "-Infinity");
assert.sameValue(Number.prototype.toFixed.call(Infinity), "Infinity");
assert.sameValue(Number.prototype.toFixed.call(NaN), "NaN");

assert.throws(RangeError, () => Number.prototype.toFixed.call(-Infinity, 555));
assert.throws(RangeError, () => Number.prototype.toFixed.call(Infinity, 555));
assert.throws(RangeError, () => Number.prototype.toFixed.call(NaN, 555));

assert.throws(TypeError, () => Number.prototype.toFixed.call("Hello"));

