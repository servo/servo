// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - value of 'length' is an Object which has an
    own valueOf method
---*/

function callbackfn(val, idx, obj) {
  return val < 10;
}

var obj = {
  0: 11,
  1: 9,

  length: {
    valueOf: function() {
      return 2;
    }
  }
};

var newArr = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
