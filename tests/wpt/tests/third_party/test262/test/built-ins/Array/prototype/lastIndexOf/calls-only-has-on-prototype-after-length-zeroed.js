// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
  Calls [[HasProperty]] on the prototype to check for existing elements.
info: |
  22.1.3.15 Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  4. If fromIndex is present, let n be ? ToInteger(fromIndex); else let n be len-1.
  ...
  7. Repeat, while k ≥ 0
    a. Let kPresent be ? HasProperty(O, ! ToString(k)).
    b. If kPresent is true, then
      i. Let elementK be ? Get(O, ! ToString(k)).
      ...
includes: [proxyTrapsHelper.js]
features: [Proxy]
---*/

var array = [5, undefined, 7];

Object.setPrototypeOf(array, new Proxy(Array.prototype, allowProxyTraps({
    has: function(t, pk) {
        return pk in t;
    }
})));

var fromIndex = {
    valueOf: function() {
        // Zero the array's length. The loop in step 8 iterates over the original
        // length value of 100, but the only prototype MOP method which should be
        // called is [[HasProperty]].
        array.length = 0;
        return 2;
    }
};

Array.prototype.lastIndexOf.call(array, 100, fromIndex);
