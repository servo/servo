// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-12-s
description: >
    Strict Mode - Eval code is strict eval code with a Use Strict
    Directive in the middle of the block
flags: [noStrict]
---*/

        eval("var public = 1; 'use strict'; var anotherVariableNotReserveWord = 2;");

assert.sameValue(public, 1, 'public');
assert.sameValue(anotherVariableNotReserveWord, 2, 'anotherVariableNotReserveWord');
