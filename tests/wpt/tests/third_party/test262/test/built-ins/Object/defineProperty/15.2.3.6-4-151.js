// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-151
description: >
    Object.defineProperty - 'O' is an Array, 'name' is the length
    property of 'O', and the [[Value]] field of 'desc' is an Object
    with an own toString method and an inherited valueOf method
    (15.4.5.1 step 3.c), test that the inherited valueOf method is used
---*/

var arrObj = [];
var toStringAccessed = false;
var valueOfAccessed = false;

var proto = {
  valueOf: function() {
    valueOfAccessed = true;
    return 2;
  }
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
child.toString = function() {
  toStringAccessed = true;
  return 3;
};

Object.defineProperty(arrObj, "length", {
  value: child
});

assert.sameValue(arrObj.length, 2, 'arrObj.length');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
assert(valueOfAccessed, 'valueOfAccessed !== true');
