// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-39
description: >
    Object.defineProperty - 'O' is a Date object that uses Object's
    [[GetOwnProperty]] method to access the 'name' property (8.12.9
    step 1)
---*/

var desc = new Date(0);

Object.defineProperty(desc, "foo", {
  value: 12,
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperty(desc, "foo", {
    value: 11,
    configurable: true
  });
});
assert.sameValue(desc.foo, 12, 'desc.foo');
