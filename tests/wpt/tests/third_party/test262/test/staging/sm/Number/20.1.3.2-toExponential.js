/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js]
description: |
  Number.prototype.toExponential
info: bugzilla.mozilla.org/show_bug.cgi?id=818617
esid: pending
---*/

// With NaN, fractionDigits out of range.
assert.sameValue(Number.prototype.toExponential.call(NaN, 555), 'NaN');

// With NaN fractionDigits in range.
assert.sameValue(Number.prototype.toExponential.call(NaN, 5), 'NaN');

// With Infinity, fractionDigits out of range.
assert.sameValue(Number.prototype.toExponential.call(Infinity, 555), 'Infinity');

// With Infinity, fractionDigits in range.
assert.sameValue(Number.prototype.toExponential.call(Infinity, 5), 'Infinity');

// With -Infinity, fractionDigits out of range.
assert.sameValue(Number.prototype.toExponential.call(-Infinity, 555), '-Infinity');

// With -Infinity, fractionDigits in range.
assert.sameValue(Number.prototype.toExponential.call(-Infinity, 5), '-Infinity');

// With NaN, function assigning a value.
let x = 10;
assert.sameValue(Number.prototype.toExponential.call(NaN, { valueOf() { x = 20; return 1; } }), 'NaN');
assert.sameValue(x, 20);

// With NaN, function throwing an exception.
assertThrowsValue(
  () => Number.prototype.toExponential.call(NaN, { valueOf() { throw "hello"; } }),
  "hello");

// Not a number throws TypeError
assert.throws(TypeError, () => Number.prototype.toExponential.call("Hello"));
