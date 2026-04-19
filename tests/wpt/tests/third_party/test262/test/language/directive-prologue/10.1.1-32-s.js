// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-32-s
description: >
    Strict Mode - Function code of built-in Function constructor
    contains Use Strict Directive which appears at the end of the block
flags: [noStrict]
---*/


        var funObj = new Function("a", "eval('public = 1;'); anotherVariable = 2; 'use strict';");
        funObj();

assert.sameValue(public, 1, 'public');
assert.sameValue(anotherVariable, 2, 'anotherVariable');
