// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-2
description: >
    Object.getOwnPropertyDescriptor returns undefined for null
    property name
---*/

var o = {};
var desc = Object.getOwnPropertyDescriptor(o, null);

assert.sameValue(desc, undefined, 'desc');
