// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-112
description: >
    Object.defineProperties - 'O' is an Array, test the length
    property of 'O' is own data property (15.4.5.1 step 1)
---*/

var arr = [0, 1];
Object.defineProperty(arr, "1", {
  value: 1,
  configurable: false
});
assert.throws(TypeError, function() {
  Object.defineProperties(arr, {
    length: {
      value: 1
    }
  });
});
var desc = Object.getOwnPropertyDescriptor(arr, "length");

assert.sameValue(desc.value, 2, 'desc.value');
assert(desc.writable, 'desc.writable !== true');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, false, 'desc.configurable');
