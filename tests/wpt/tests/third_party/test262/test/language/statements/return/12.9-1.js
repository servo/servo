// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.9-1
description: >
    The return Statement - a return statement without an expression
    may have a LineTerminator before the semi-colon
---*/

        var sum = 0;
        (function innerTest() {
            for (var i = 1; i <= 10; i++) {
                if (i === 6) {
                    return
                    ;
                }
                sum += i;
            }
        })();

assert.sameValue(sum, 15, 'sum');
