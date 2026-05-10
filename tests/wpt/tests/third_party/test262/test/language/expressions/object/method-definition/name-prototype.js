// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    The prototype of functions declared as methods is the Function prototype.
es6id: 14.3.8
---*/

var obj = { method() {} };
assert.sameValue(Object.getPrototypeOf(obj.method), Function.prototype);
