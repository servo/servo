// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.filter
description: Constructor is ignored for non-Array values
info: |
    [...]
    5. Let A be ? ArraySpeciesCreate(O, 0).
    [...]

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

result = Array.prototype.filter.call(obj, function() {});

assert.sameValue(callCount, 0, '`constructor` property not accessed');
assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert(Array.isArray(result), 'result is an Array exotic object');
assert.sameValue(result.length, 0, 'array created with appropriate length');
