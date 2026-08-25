// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - element to be retrieved is own data property
    that overrides an inherited accessor property on an Array-like
    object
---*/

var kValue = "abc";

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    return val === kValue;
  }
  return false;
}

var proto = {};

Object.defineProperty(proto, "5", {
  get: function() {
    return 11;
  },
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;
Object.defineProperty(child, "5", {
  value: kValue,
  configurable: true
});

var testResult = Array.prototype.map.call(child, callbackfn);

assert.sameValue(testResult[5], true, 'testResult[5]');
