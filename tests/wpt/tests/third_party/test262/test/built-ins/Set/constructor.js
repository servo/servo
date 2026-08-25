// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-constructor
description: >
    The Set constructor is the %Set% intrinsic object and the
    initial value of the Set property of the global object.
---*/

assert.sameValue(typeof Set, "function", "`typeof Set` is `'function'`");
