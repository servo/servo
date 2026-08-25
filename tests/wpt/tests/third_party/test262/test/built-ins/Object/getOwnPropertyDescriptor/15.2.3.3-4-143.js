// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-143
description: >
    Object.getOwnPropertyDescriptor returns data desc for functions on
    built-ins (Date.prototype.setSeconds)
---*/

var desc = Object.getOwnPropertyDescriptor(Date.prototype, "setSeconds");

assert.sameValue(desc.value, Date.prototype.setSeconds, 'desc.value');
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
