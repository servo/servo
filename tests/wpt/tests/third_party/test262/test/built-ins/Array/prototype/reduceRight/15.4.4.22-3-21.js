// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - 'length' is an object that has an
    own valueOf method that returns an object and toString method that
    returns a string
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
var valueOfAccessed = false;

var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: {
    valueOf: function() {
      valueOfAccessed = true;
      return {};
    },
    toString: function() {
      toStringAccessed = true;
      return '2';
    }
  }
};

Array.prototype.reduceRight.call(obj, callbackfn, 1);

assert(testResult1, 'testResult1 !== true');
assert(testResult2, 'testResult2 !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert(toStringAccessed, 'toStringAccessed !== true');
