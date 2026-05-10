// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.1.5.1
description: The `length` property of GetCapabilitiesExecutor functions
info: |
  The length property of a GetCapabilitiesExecutor function is 2.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var executorFunction;

function NotPromise(executor) {
  executorFunction = executor;
  executor(function() {}, function() {});
}
Promise.resolve.call(NotPromise);

verifyProperty(executorFunction, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
