// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf uses inherited valueOf method when
    'length' is an object with an own toString and an inherited
    valueOf methods
---*/

var toStringAccessed = false;
var valueOfAccessed = false;

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
  return 2;
};

var obj = {
  1: child,
  length: child
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, child), 1, 'Array.prototype.lastIndexOf.call(obj, child)');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
