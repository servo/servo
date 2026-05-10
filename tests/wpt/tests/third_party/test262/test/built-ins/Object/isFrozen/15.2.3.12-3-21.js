// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-3-21
description: Object.isFrozen returns false for all built-in objects (EvalError)
---*/

var b = Object.isFrozen(EvalError);

assert.sameValue(b, false, 'b');
