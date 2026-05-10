// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - value of 'length' is an object which
    has an own toString method
---*/

var testResult1 = true;
var testResult2 = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx > 1) {
    testResult1 = false;
  }

  if (idx === 1) {
    testResult2 = true;
  }
  return false;
}

var toStringAccessed = false;
var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: {
    toString: function() {
      toStringAccessed = true;
      return '2';
    }
  }
};

// objects inherit the default valueOf() method from Object
// that simply returns itself. Since the default valueOf() method
// does not return a primitive value, ES next tries to convert the object
// to a number by calling its toString() method and converting the
// resulting string to a number.

Array.prototype.reduceRight.call(obj, callbackfn, 1);

assert(testResult1, 'testResult1 !== true');
assert(testResult2, 'testResult2 !== true');
assert(toStringAccessed, 'toStringAccessed !== true');
