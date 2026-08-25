// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some uses inherited valueOf method when 'length'
    is an object with an own toString and inherited valueOf methods
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var valueOfAccessed = false;
var toStringAccessed = false;

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
  0: 9,
  1: 11,
  2: 12,
  length: child
};

assert(Array.prototype.some.call(obj, callbackfn1), 'Array.prototype.some.call(obj, callbackfn1) !== true');
assert.sameValue(Array.prototype.some.call(obj, callbackfn2), false, 'Array.prototype.some.call(obj, callbackfn2)');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
