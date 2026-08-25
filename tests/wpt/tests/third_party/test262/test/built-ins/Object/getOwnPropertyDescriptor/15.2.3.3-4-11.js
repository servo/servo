// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-11
description: >
    Object.getOwnPropertyDescriptor returns data desc for functions on
    built-ins (Global.encodeURIComponent)
---*/

var global = this;
var desc = Object.getOwnPropertyDescriptor(global, "encodeURIComponent");

assert.sameValue(desc.value, global.encodeURIComponent, 'desc.value');
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
