// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is inherited
    accessor property without a get function on an Array-like object
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    testResult = (typeof val === "undefined");
  }
}

var proto = {};
Object.defineProperty(proto, "1", {
  set: function() {},
  configurable: true
});

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 2;

Array.prototype.forEach.call(child, callbackfn);

assert(testResult, 'testResult !== true');
