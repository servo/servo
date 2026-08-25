// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.4.5.1-5-1
description: >
    Defining a property named 4294967295 (2**32-1)(not an array
    element)
---*/

var a = [];
a[4294967295] = "not an array element";

assert.sameValue(a[4294967295], "not an array element", 'The value of a[4294967295] is expected to be "not an array element"');
