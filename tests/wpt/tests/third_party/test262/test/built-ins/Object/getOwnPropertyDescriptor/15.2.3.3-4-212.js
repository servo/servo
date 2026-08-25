// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-212
description: >
    Object.getOwnPropertyDescriptor returns accessor desc for
    accessors on built-ins (RegExp.prototype.source)
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, "source");

assert.sameValue(desc.hasOwnProperty('writable'), false, 'desc.hasOwnProperty("writable")');
assert.sameValue(desc.enumerable, false, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
assert.sameValue(typeof desc.get, 'function', 'typeof desc.get');
assert.sameValue(desc.set, undefined, 'desc.set');
