// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.2
description: The `length` property of Promise.all Resolve Element functions
info: |
  The length property of a Promise.all resolve element function is 1.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var resolveElementFunction;
var thenable = {
  then: function(fulfill) {
    resolveElementFunction = fulfill;
  }
};

function NotPromise(executor) {
  executor(function() {}, function() {});
}
NotPromise.resolve = function(v) {
  return v;
};
Promise.all.call(NotPromise, [thenable]);

verifyProperty(resolveElementFunction, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
