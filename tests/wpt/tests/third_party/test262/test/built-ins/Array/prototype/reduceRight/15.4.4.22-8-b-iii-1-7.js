// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is inherited
    data property on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === 2);
  }
}

var proto = {
  0: 0,
  1: 1,
  2: 2,
  length: 3
};
var Con = function() {};
Con.prototype = proto;

var child = new Con();

Array.prototype.reduceRight.call(child, callbackfn);

assert(testResult, 'testResult !== true');
