// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-554
description: >
    ES5 Attributes - success to update [[Enumerable]] attribute of
    accessor property ([[Get]] is a Function, [[Set]] is a Function,
    [[Enumerable]] is false, [[Configurable]] is true) to different
    value
---*/

var obj = {};

var getFunc = function() {
  return 1001;
};

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: setFunc,
  enumerable: false,
  configurable: true
});

var result1 = false;
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p1 in obj) {
  if (p1 === "prop") {
    result1 = true;
  }
}

Object.defineProperty(obj, "prop", {
  enumerable: true
});
var result2 = false;
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");
for (var p2 in obj) {
  if (p2 === "prop") {
    result2 = true;
  }
}

assert.sameValue(result1, false, 'result1');
assert(result2, 'result2 !== true');
assert.sameValue(desc1.enumerable, false, 'desc1.enumerable');
assert.sameValue(desc2.enumerable, true, 'desc2.enumerable');
