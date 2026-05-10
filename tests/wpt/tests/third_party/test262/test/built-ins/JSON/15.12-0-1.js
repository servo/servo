// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This test should be run without any built-ins being added/augmented.
    The name JSON must be bound to an object.
    4.2 calls out JSON as one of the built-in objects.
es5id: 15.12-0-1
description: JSON must be a built-in object
---*/

var o = JSON;

assert.sameValue(typeof(o), "object", 'typeof(o)');
