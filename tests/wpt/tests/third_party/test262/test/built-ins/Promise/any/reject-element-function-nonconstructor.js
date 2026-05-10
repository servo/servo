// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any-reject-element-functions
description: Promise.any Reject Element functions are not constructors
info: |
  17 ECMAScript Standard Built-in Objects:
    Built-in function objects that are not identified as constructors do not
    implement the [[Construct]] internal method unless otherwise specified
    in the description of a particular function.
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

assert.sameValue(Object.prototype.hasOwnProperty.call(rejectElementFunction, 'prototype'), false);
assert.throws(TypeError, function() {
  new rejectElementFunction();
});
