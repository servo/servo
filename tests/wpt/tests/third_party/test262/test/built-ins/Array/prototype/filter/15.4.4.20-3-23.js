// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter uses inherited valueOf method when 'length'
    is an object with an own toString and inherited valueOf methods
---*/

var valueOfAccessed = false;
var toStringAccessed = false;

function callbackfn(val, idx, obj) {
  return true;
}

var proto = {
  valueOf: function() {
    valueOfAccessed = true;
    return 2;
  }
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();

child.toString = function() {
  toStringAccessed = true;
  return '1';
};

var obj = {
  1: 11,
  2: 9,
  length: child
};

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
