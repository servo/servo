// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-54-s
description: >
    checking 'this' (Literal getter)
---*/

var o = { get foo() { return this; } }

assert.sameValue(o.foo, o, 'o.foo');
