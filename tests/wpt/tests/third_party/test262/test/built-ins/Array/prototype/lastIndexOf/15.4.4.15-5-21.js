// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' which is an
    Object, and has an own toString method
---*/

// objects inherit the default valueOf() method from Object
// that simply returns itself. Since the default valueOf() method
// does not return a primitive value, ES next tries to convert the object
// to a number by calling its toString() method and converting the
// resulting string to a number.
var fromIndex = {
  toString: function() {
    return '2';
  }
};
var targetObj = new RegExp();

assert.sameValue([0, true, targetObj, 3, false].lastIndexOf(targetObj, fromIndex), 2, '[0, true, targetObj, 3, false].lastIndexOf(targetObj, fromIndex)');
assert.sameValue([0, true, 3, targetObj, false].lastIndexOf(targetObj, fromIndex), -1, '[0, true, 3, targetObj, false].lastIndexOf(targetObj, fromIndex)');
