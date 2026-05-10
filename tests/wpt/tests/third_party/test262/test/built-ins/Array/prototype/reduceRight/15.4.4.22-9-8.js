// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - no observable effects occur if 'len'
    is 0
---*/

var accessed = false;
var callbackAccessed = false;

function callbackfn() {
  callbackAccessed = true;
}

var obj = {
  length: 0
};

Object.defineProperty(obj, "5", {
  get: function() {
    accessed = true;
    return 10;
  },
  configurable: true
});

Array.prototype.reduceRight.call(obj, callbackfn, "initialValue");

assert.sameValue(accessed, false, 'accessed');
assert.sameValue(callbackAccessed, false, 'callbackAccessed');
