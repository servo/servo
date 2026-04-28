// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-11-s
description: >
    Strict Mode - Eval code is strict code with a Use Strict Directive
    at the beginning of the block
flags: [noStrict]
---*/

var err = null;

        try {
            eval("'use strict'; var public = 1; var anotherVariableNotReserveWord = 2;");
        } catch (e) {
            err = e;
        }

assert(err instanceof SyntaxError, 'err instanceof SyntaxError');
assert.sameValue(typeof public, "undefined", 'typeof public');
assert.sameValue(typeof anotherVariableNotReserveWord, "undefined", 'typeof anotherVariableNotReserveWord');
