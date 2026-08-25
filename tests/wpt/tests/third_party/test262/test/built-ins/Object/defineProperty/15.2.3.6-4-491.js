// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-491
description: >
    ES5 Attributes - fail to update [[Enumerable]] attribute of
    accessor property ([[Get]] is undefined, [[Set]] is a Function,
    [[Enumerable]] is false, [[Configurable]] is false) to different
    value
---*/

var obj = {};

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: setFunc,
  enumerable: false,
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
    enumerable: true
  });
});
var result2 = false;
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p2 in obj) {
  if (p2 === "prop") {
    result2 = true;
  }
}

assert.sameValue(result1, false, 'result1');
assert.sameValue(result2, false, 'result2');
assert.sameValue(desc1.enumerable, false, 'desc1.enumerable');
assert.sameValue(desc2.enumerable, false, 'desc2.enumerable');
