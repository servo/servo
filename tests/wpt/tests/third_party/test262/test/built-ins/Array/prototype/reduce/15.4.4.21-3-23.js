// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce uses inherited valueOf method - 'length' is
    an object with an own toString and inherited valueOf methods
---*/

var valueOfAccessed = false;
var toStringAccessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  return (curVal === 11 && idx === 1);
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

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, 1), true, 'Array.prototype.reduce.call(obj, callbackfn, 1)');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
