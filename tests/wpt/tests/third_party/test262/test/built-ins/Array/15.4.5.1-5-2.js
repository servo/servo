// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.4.5.1-5-2
description: >
    Defining a property named 4294967295 (2**32-1) doesn't change
    length of the array
---*/

var a = [0, 1, 2];
a[4294967295] = "not an array element";

assert.sameValue(a.length, 3, 'The value of a.length is expected to be 3');
