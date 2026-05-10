// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.1.3.2
description: Promise Resolve functions are not constructors
info: |
  17 ECMAScript Standard Built-in Objects:
    Built-in function objects that are not identified as constructors do not
    implement the [[Construct]] internal method unless otherwise specified
    in the description of a particular function.
---*/

var resolveFunction;
new Promise(function(resolve, reject) {
  resolveFunction = resolve;
});

assert.sameValue(Object.prototype.hasOwnProperty.call(resolveFunction, "prototype"), false);
assert.throws(TypeError, function() {
  new resolveFunction();
});
