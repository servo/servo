// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own data
    property that overrides an inherited data property on an
    Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === "1");
  }
}

var proto = {
  0: 10,
  1: 11,
  2: 12,
  length: 2
};
var Con = function() {};
Con.prototype = proto;

var child = new Con();
child[1] = "1";
child[2] = "2";
child.length = 3;

Array.prototype.reduceRight.call(child, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
