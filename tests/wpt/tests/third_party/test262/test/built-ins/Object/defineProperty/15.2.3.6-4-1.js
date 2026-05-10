// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Step 4 of defineProperty calls the [[DefineOwnProperty]] internal method
    of O passing 'true' for the Throw flag. In this case, step 3 of
    [[DefineOwnProperty]] requires that it throw a TypeError exception when
    current is undefined and extensible is false. The value of desc does not
    matter.
es5id: 15.2.3.6-4-1
description: >
    Object.defineProperty throws TypeError when adding properties to
    non-extensible objects(8.12.9 step 3)
---*/

var o = {};
Object.preventExtensions(o);
assert.throws(TypeError, function() {
  var desc = {
    value: 1
  };
  Object.defineProperty(o, "foo", desc);
});
assert.sameValue(o.hasOwnProperty("foo"), false, 'o.hasOwnProperty("foo")');
