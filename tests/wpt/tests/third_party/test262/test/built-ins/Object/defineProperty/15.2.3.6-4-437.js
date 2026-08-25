// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-437
description: >
    ES5 Attributes - fail to update [[Enumerable]] attribute of
    accessor property ([[Get]] is undefined, [[Set]] is undefined,
    [[Enumerable]] is true, [[Configurable]] is false) to different
    value
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
  configurable: false
});
var result1 = false;
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p1 in obj) {
  if (p1 === "prop") {
    result1 = true;
  }
}
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "prop", {
    enumerable: false
  });
});
var result2 = false;
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p2 in obj) {
  if (p2 === "prop") {
    result2 = true;
  }
}

assert(result1, 'result1 !== true');
assert(result2, 'result2 !== true');
assert.sameValue(desc1.enumerable, true, 'desc1.enumerable');
assert.sameValue(desc2.enumerable, true, 'desc2.enumerable');
