// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.slice
description: >
    Ensure a RangeError is thrown when a proxied array returns an invalid array length.
info: |
    [...]
    8. Let A be ? ArraySpeciesCreate(O, count).
    [...]

    9.4.2.3 ArraySpeciesCreate

    [...]
    3. Let isArray be ? IsArray(originalArray).
    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    10. Return ? Construct(C, « length »).

    9.4.2.2 ArrayCreate

    [...]
    3. If length>2^32-1, throw a RangeError exception.
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
  Array.prototype.slice.call(proxy);
});

assert.sameValue(
  callCount,
  0,
  'RangeError thrown during array creation, not property modification'
);
