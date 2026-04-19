// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-4-1
description: Object.isFrozen returns false if extensible is true
---*/

assert.sameValue(Object.isFrozen({}), false, 'Object.isFrozen({})');
