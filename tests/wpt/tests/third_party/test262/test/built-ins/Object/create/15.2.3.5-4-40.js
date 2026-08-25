// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-40
description: >
    Object.create - ensure that if an exception is thrown it occurs in
    the correct order relative to prior and subsequent side-effects
    (15.2.3.7 step 5.a)
---*/

var newObj = {};
var props = {};
var i = 0;

Object.defineProperty(props, "prop1", {
  get: function() {
    i++;
    return {};
  },
  enumerable: true
});

Object.defineProperty(props, "prop2", {
  get: function() {
    if (1 === i++) {
      throw new RangeError();
    } else {
      return {};
    }
  },
  enumerable: true
});
assert.throws(RangeError, function() {
  newObj = Object.create({}, props);
});
assert.sameValue(newObj.hasOwnProperty("prop1"), false, 'newObj.hasOwnProperty("prop1")');
assert.sameValue(i, 2, 'i');
