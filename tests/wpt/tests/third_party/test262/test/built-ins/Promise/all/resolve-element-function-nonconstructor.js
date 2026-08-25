// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.1.2
description: Promise.all Resolve Element functions are not constructors
info: |
  17 ECMAScript Standard Built-in Objects:
    Built-in function objects that are not identified as constructors do not
    implement the [[Construct]] internal method unless otherwise specified
    in the description of a particular function.
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

assert.sameValue(Object.prototype.hasOwnProperty.call(resolveElementFunction, "prototype"), false);
assert.throws(TypeError, function() {
  new resolveElementFunction();
});
