// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled-reject-element-functions
description: Promise.allSettled Reject Element functions are not constructors
info: |
  17 ECMAScript Standard Built-in Objects:
    Built-in function objects that are not identified as constructors do not
    implement the [[Construct]] internal method unless otherwise specified
    in the description of a particular function.
features: [Promise.allSettled]
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
Promise.allSettled.call(NotPromise, [thenable]);

assert.sameValue(Object.prototype.hasOwnProperty.call(rejectElementFunction, 'prototype'), false);
assert.throws(TypeError, function() {
  new rejectElementFunction();
});
