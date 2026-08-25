// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.splice
description: >
    An undefined value for the @@species constructor triggers the creation  of
    an Array exotic object
info: |
    [...]
    9. Let A be ? ArraySpeciesCreate(O, actualDeleteCount).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    7. If Type(C) is Object, then
       a. Let C be ? Get(C, @@species).
       b. If C is null, let C be undefined.
    8. If C is undefined, return ? ArrayCreate(length).

    9.4.2.2 ArrayCreate

    [...]
    3. If length>232-1, throw a RangeError exception.
features: [Proxy]
---*/

var array = [];
var maxLength = Math.pow(2, 32);
var callCount = 0;
var proxy = new Proxy(array, {
  get: function(_, name) {
    if (name === 'length') {
      return maxLength;
    }
    return array[name];
  },
  set: function() {
    callCount += 1;
    return true;
  }
});

assert.throws(RangeError, function() {
  Array.prototype.splice.call(proxy, 0);
});

assert.sameValue(
  callCount,
  0,
  'RangeError thrown during array creation, not property modification'
);
