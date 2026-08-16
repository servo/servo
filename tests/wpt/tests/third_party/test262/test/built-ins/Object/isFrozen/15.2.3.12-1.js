// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-1
description: >
    Object.isFrozen does not throw TypeError if type of first param is
    not Object
---*/

assert.sameValue(Object.isFrozen(0), true);
