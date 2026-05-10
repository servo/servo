// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-144
description: >
    Object.defineProperties -  'O' is an Array, 'name' is the length
    property of 'O', test the [[Value]] field of 'desc' is an Object
    which has an own valueOf method that returns an object and
    toString method that returns a string (15.4.5.1 step 3.c)
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
        return {};
      }
    }
  }
});

assert.sameValue(arr.length, 2, 'arr.length');
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
