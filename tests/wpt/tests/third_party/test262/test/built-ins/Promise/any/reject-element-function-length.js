// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any-reject-element-functions
description: The `length` property of Promise.any Reject Element functions
info: |
  The length property of a Promise.any Reject Element function is 1.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Promise.any]
---*/

var rejectElementFunction;
var thenable = {
  then(_, reject) {
    rejectElementFunction = reject;
  }
};

function NotPromise(executor) {
  executor(function() {}, function() {});
}
NotPromise.resolve = function(v) {
  return v;
};
Promise.any.call(NotPromise, [thenable]);

assert.sameValue(rejectElementFunction.length, 1);

verifyProperty(rejectElementFunction, 'length', {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true,
});
