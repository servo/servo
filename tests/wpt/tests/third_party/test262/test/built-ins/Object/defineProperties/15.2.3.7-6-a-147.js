// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-147
description: >
    Object.defineProperties - 'O' is an Array, 'name' is the length
    property of 'O', test using inherited valueOf method when the
    [[Value]] field of 'desc' is an Objec with an own toString and
    inherited valueOf methods (15.4.5.1 step 3.c)
---*/

var arr = [];
var toStringAccessed = false;
var valueOfAccessed = false;

var proto = {
  value: {
    valueOf: function() {
      valueOfAccessed = true;
      return 2;
    }
  }
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
Object.defineProperty(child, "value", {
  value: {
    toString: function() {
      toStringAccessed = true;
      return 3;
    }
  }
});

Object.defineProperties(arr, {
  length: child
});

assert.sameValue(arr.length, 3, 'arr.length');
assert(toStringAccessed, 'toStringAccessed !== true');
assert.sameValue(valueOfAccessed, false, 'valueOfAccessed');
