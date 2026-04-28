// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element to be retrieved is own data
    property that overrides an inherited data property on an
    Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === "9");
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
child[0] = "9";
child[1] = "1";
child.length = 3;

Array.prototype.reduce.call(child, callbackfn);

assert(testResult, 'testResult !== true');
