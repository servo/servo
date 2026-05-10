// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.map
description: >
    An undefined value for the @@species constructor triggers the creation  of
    an Array exotic object
info: |
    [...]
    5. Let A be ? ArraySpeciesCreate(O, len).
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
var cbCount = 0;
var setCount = 0;
var cb = function() {
  cbCount += 1;
};
var proxy = new Proxy(array, {
  get: function(_, name) {
    if (name === 'length') {
      return maxLength;
    }
    return array[name];
  },
  set: function() {
    setCount += 1;
    return true;
  }
});

assert.throws(RangeError, function() {
  Array.prototype.map.call(proxy, cb);
});

assert.sameValue(
  setCount,
  0,
  'RangeError thrown during array creation, not property modification'
);
assert.sameValue(cbCount, 0, 'callback function not invoked');
