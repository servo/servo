// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: returns false if the element is not found
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

    ...
  5. If n ≥ 0, then
    a. Let k be n.
  6. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  8. Return false.
features: [Symbol, Array.prototype.includes]
---*/

assert.sameValue([42].includes(43), false, "43");

assert.sameValue(["test262"].includes("test"), false, "string");

assert.sameValue([0, "test262", undefined].includes(""), false, "the empty string");

assert.sameValue(["true", false].includes(true), false, "true");
assert.sameValue(["", true].includes(false), false, "false");

assert.sameValue([undefined, false, 0, 1].includes(null), false, "null");
assert.sameValue([null].includes(undefined), false, "undefined");

assert.sameValue([Symbol("1")].includes(Symbol("1")), false, "symbol");
assert.sameValue([{}].includes({}), false, "object");
assert.sameValue([
  []
].includes([]), false, "array");

var sample = [42];
assert.sameValue(sample.includes(sample), false, "this");
