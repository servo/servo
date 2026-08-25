/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// ES6 does not specify enumeration order, but implementations mostly retain
// property insertion order -- and must, for web compatibility. This test checks
// that symbol-keyed properties do not interfere with that order.

var obj = {};
obj[Symbol("moon")] = 0;
obj.x = 1;
obj[Symbol.for("y")] = 2
obj.y = 3;
obj[Symbol.iterator] = function* () { yield 4; };
obj.z = 5;
Object.prototype[Symbol.for("comet")] = 6;

var keys = [];
for (var k in obj)
    keys.push(k);
assert.compareArray(keys, ["x", "y", "z"]);
assert.compareArray(Object.keys(obj), ["x", "y", "z"]);

// Test with more properties.
for (var i = 0; i < 1000; i++)
    obj[Symbol(i)] = i;
obj.w = 1000;
keys = []
for (var k in obj)
    keys.push(k);
assert.compareArray(keys, ["x", "y", "z", "w"]);
