// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-3-13
description: >
    Object.getOwnPropertyDescriptor applied to the Arguments object
    which implements its own property get method
---*/

var arg = (function() {
  return arguments;
}("ownProperty", true));

var desc = Object.getOwnPropertyDescriptor(arg, "0");

assert.sameValue(desc.value, "ownProperty", 'desc.value');
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
