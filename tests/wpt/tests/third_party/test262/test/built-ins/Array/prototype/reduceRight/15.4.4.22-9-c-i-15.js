// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is inherited
    accessor property on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === 1);
  }
}

var proto = {
  0: 0,
  2: 2
};

Object.defineProperty(proto, "1", {
  get: function() {
    return 1;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 3;

Array.prototype.reduceRight.call(child, callbackfn, "initialValue");

assert(testResult, 'testResult !== true');
