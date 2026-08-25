// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every uses inherited valueOf method when 'length'
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

Object.defineProperty(child, "toString", {
  value: function() {
    toStringAccessed = true;
    return '1';
  }
});

var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: child
};

assert(Array.prototype.every.call(obj, callbackfn1), 'Array.prototype.every.call(obj, callbackfn1) !== true');
assert.sameValue(Array.prototype.every.call(obj, callbackfn2), false, 'Array.prototype.every.call(obj, callbackfn2)');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
