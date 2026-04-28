// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-145
description: >
    Object.defineProperties - 'O' is an Array, 'name' is the length
    property of 'O', test the [[Value]] field of 'desc' is an Object
    which has an own toString and valueOf method (15.4.5.1 step 3.c)
---*/

var arr = [];
var toStringAccessed = false;
var valueOfAccessed = false;

Object.defineProperties(arr, {
  length: {
    value: {
      toString: function() {
        toStringAccessed = true;
        return '2';
      },

      valueOf: function() {
        valueOfAccessed = true;
        return 3;
      }
    }
  }
});

assert.sameValue(arr.length, 3, 'arr.length');
assert.sameValue(toStringAccessed, false, 'toStringAccessed');
assert(valueOfAccessed, 'valueOfAccessed !== true');
