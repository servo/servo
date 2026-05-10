// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.1.3.1
description: The `length` property of Promise Reject functions
info: |
  The length property of a promise reject function is 1.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var rejectFunction;
new Promise(function(resolve, reject) {
  rejectFunction = reject;
});

verifyProperty(rejectFunction, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
