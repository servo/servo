// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-113
description: >
    Object.defineProperties - 'O' is an Array, test the length
    property of 'O' is own data property that overrides an inherited
    data property (15.4.5.1 step 1)
---*/

var arrProtoLen;
var arr = [0, 1, 2];

assert.throws(TypeError, function() {
  arrProtoLen = Array.prototype.length;
  Array.prototype.length = 0;

  Object.defineProperty(arr, "2", {
    configurable: false
  });

  Object.defineProperties(arr, {
    length: {
      value: 1
    }
  });
});

var desc = Object.getOwnPropertyDescriptor(arr, "length");
assert.sameValue(desc.value, 3, 'desc.value');
assert(desc.writable, 'desc.writable !== true');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, false, 'desc.configurable');
