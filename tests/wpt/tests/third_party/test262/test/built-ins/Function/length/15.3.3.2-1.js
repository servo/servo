// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.3.2-1
description: Function.length - data property with value 1
---*/

var desc = Object.getOwnPropertyDescriptor(Function, "length");

assert.sameValue(desc.value, 1, 'desc.value');
assert.sameValue(desc.writable, false, 'desc.writable');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
