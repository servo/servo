// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.splice
description: Abrupt completion from creating a new array
info: |
    [...]
    9. Let A be ? ArraySpeciesCreate(O, actualDeleteCount).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    3. Let isArray be ? IsArray(originalArray).
    4. If isArray is false, return ? ArrayCreate(length).

    9.4.2.2 ArrayCreate

    [...]
    3. If length>232-1, throw a RangeError exception.
---*/

var callCount = 0;
var maxLength = Math.pow(2, 32);
var obj = Object.defineProperty({}, 'length', {
  get: function() {
    return maxLength;
  },
  set: function() {
    callCount += 1;
  }
});

assert.throws(RangeError, function() {
  Array.prototype.splice.call(obj, 0);
});

assert.sameValue(
  callCount,
  0,
  'RangeError thrown during array creation, not property modification'
);
