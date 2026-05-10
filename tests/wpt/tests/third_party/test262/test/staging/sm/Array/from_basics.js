/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// Array.from copies arrays.
var src = [1, 2, 3], copy = Array.from(src);
assert.sameValue(copy === src, false);
assert.sameValue(Array.isArray(copy), true);
assert.deepEqual(copy, src);

// Non-element properties are not copied.
var a = [0, 1];
a.name = "lisa";
assert.deepEqual(Array.from(a), [0, 1]);

// It's a shallow copy.
src = [[0], [1]];
copy = Array.from(src);
assert.sameValue(copy[0], src[0]);
assert.sameValue(copy[1], src[1]);

// Array.from can copy non-iterable objects, if they're array-like.
src = {0: "zero", 1: "one", length: 2};
copy = Array.from(src);
assert.sameValue(Array.isArray(copy), true);
assert.deepEqual(copy, ["zero", "one"]);

// Properties past the .length are not copied.
src = {0: "zero", 1: "one", 2: "two", 9: "nine", name: "lisa", length: 2};
assert.deepEqual(Array.from(src), ["zero", "one"]);

// If an object has neither an @@iterator method nor .length,
// then it's treated as zero-length.
assert.deepEqual(Array.from({}), []);

// Source object property order doesn't matter.
src = {length: 2, 1: "last", 0: "first"};
assert.deepEqual(Array.from(src), ["first", "last"]);

// Array.from does not preserve holes.
assert.deepEqual(Array.from(Array(3)), [undefined, undefined, undefined]);
assert.deepEqual(Array.from([, , 2, 3]), [undefined, undefined, 2, 3]);
assert.deepEqual(Array.from([0, , , ,]), [0, undefined, undefined, undefined]);

// Even on non-iterable objects.
assert.deepEqual(Array.from({length: 4}), [undefined, undefined, undefined, undefined]);

// Array.from should coerce negative lengths to zero.
assert.deepEqual(Array.from({length: -1}), []);

