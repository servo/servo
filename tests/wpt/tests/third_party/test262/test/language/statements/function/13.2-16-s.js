// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-16-s
description: >
    StrictMode - enumerating over a function object looking for
    'arguments' fails inside the function
---*/

var foo = new Function("'use strict'; for (var tempIndex in this) {assert.notSameValue(tempIndex, 'arguments', 'tempIndex');}");
foo.call(foo);
