// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - side effects produced by step 3 are
    visible when an exception occurs
---*/

var stepFiveOccurs = false;

var obj = {};
Object.defineProperty(obj, "length", {
  get: function() {
    return {
      valueOf: function() {
        throw new TypeError();
      }
    };
  },
  configurable: true
});

var fromIndex = {
  valueOf: function() {
    stepFiveOccurs = true;
    return 0;
  }
};
assert.throws(TypeError, function() {
  Array.prototype.indexOf.call(obj, undefined, fromIndex);
});
assert.sameValue(stepFiveOccurs, false, 'stepFiveOccurs');
