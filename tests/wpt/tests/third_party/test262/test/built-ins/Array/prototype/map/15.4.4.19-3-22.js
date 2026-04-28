// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map throws TypeError exception when 'length' is an
    object with toString and valueOf methods that don�t return
    primitive values
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}

var obj = {
  1: 11,
  2: 12,

  length: {
    valueOf: function() {
      return {};
    },
    toString: function() {
      return {};
    }
  }
};
assert.throws(TypeError, function() {
  Array.prototype.map.call(obj, callbackfn);
});
