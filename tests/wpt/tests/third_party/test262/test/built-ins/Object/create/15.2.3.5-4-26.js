// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-26
description: >
    Object.create - TypeError is thrown when own enumerable accessor
    property of 'Properties' without a get function (15.2.3.7 step 5.a)
---*/

var props = {};
Object.defineProperty(props, "prop", {
  set: function() {},
  enumerable: true
});
assert.throws(TypeError, function() {
  Object.create({}, props);
});
