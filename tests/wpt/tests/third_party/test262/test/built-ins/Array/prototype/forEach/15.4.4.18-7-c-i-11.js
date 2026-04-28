// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is own accessor
    property that overrides an inherited data property on an
    Array-like object
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    testResult = (val === 11);
  }
}

var proto = {
  0: 5
};

var Con = function() {};
Con.prototype = proto;

var child = new Con();
child.length = 10;

Object.defineProperty(child, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

Array.prototype.forEach.call(child, callbackfn);

assert(testResult, 'testResult !== true');
