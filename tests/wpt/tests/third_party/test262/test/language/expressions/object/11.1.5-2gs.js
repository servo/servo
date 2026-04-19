// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.5-2gs
description: >
    Duplicate definitions of data properties are allowed in ObjectLiterals.
---*/

var obj = { _11_1_5_2_gs: 10, _11_1_5_2_gs: 20 };
assert.sameValue(obj._11_1_5_2_gs, 20);
