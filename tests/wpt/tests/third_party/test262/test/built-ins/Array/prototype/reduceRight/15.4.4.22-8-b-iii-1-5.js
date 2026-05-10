// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - element to be retrieved is own data
    property that overrides an inherited accessor property on an
    Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === "20");
  }
}

var proto = {};

Object.defineProperty(proto, "2", {
  get: function() {
    return 11;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 3;
child[0] = "0";
child[1] = "1";
Object.defineProperty(proto, "2", {
  value: "20",
  configurable: true
});

Array.prototype.reduceRight.call(child, callbackfn);

assert(testResult, 'testResult !== true');
