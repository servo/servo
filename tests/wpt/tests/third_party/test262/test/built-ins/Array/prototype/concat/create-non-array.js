// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.concat
description: Constructor is ignored for non-Array values
info: |
    1. Let O be ? ToObject(this value).
    2. Let A be ? ArraySpeciesCreate(O, 0).

    9.4.2.3 ArraySpeciesCreate

    [...]
    3. Let isArray be ? IsArray(originalArray).
    4. If isArray is false, return ? ArrayCreate(length).
---*/

var obj = {
  length: 0
};
var callCount = 0;
var result;
Object.defineProperty(obj, 'constructor', {
  get: function() {
    callCount += 1;
  }
});

result = Array.prototype.concat.call(obj);

assert.sameValue(callCount, 0, 'The value of callCount is expected to be 0');
assert.sameValue(
  Object.getPrototypeOf(result),
  Array.prototype,
  'Object.getPrototypeOf(Array.prototype.concat.call(obj)) returns Array.prototype'
);
assert(Array.isArray(result), 'Array.isArray(result) must return true');
assert.sameValue(result.length, 1, 'The value of result.length is expected to be 1');
assert.sameValue(result[0], obj, 'The value of result[0] is expected to equal the value of obj');
