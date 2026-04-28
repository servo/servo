// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-4-2
description: >
    Object.getOwnPropertyDescriptor returns undefined for non-existent
    properties
---*/

var o = {};

var desc = Object.getOwnPropertyDescriptor(o, "foo");

assert.sameValue(desc, undefined, 'desc');
