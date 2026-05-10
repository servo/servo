// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - element to be retrieved is own accessor
    property without a get function that overrides an inherited
    accessor property on an Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return typeof val === "undefined";
  }
  return false;
}

var proto = {};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 2;

Object.defineProperty(child, "0", {
  set: function() {},
  configurable: true
});

Object.defineProperty(proto, "0", {
  get: function() {
    return 100;
  },
  configurable: true
});

var testResult = Array.prototype.map.call(child, callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
