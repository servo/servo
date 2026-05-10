// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
es5id: 15.4.4.18-3-23
description: >
    Array.prototype.forEach uses inherited valueOf method when
    'length' is an object with an own toString and inherited valueOf
    methods
---*/

var testResult = false;
var valueOfAccessed = false;
var toStringAccessed = false;

function callbackfn(val, idx, obj) {
  testResult = (val > 10);
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

Array.prototype.forEach.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
