// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Replaces consecutive matches when the `g` flag is present
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    13. Repeat, while done is false
        a. Let result be RegExpExec(rx, S).
        b. ReturnIfAbrupt(result).
        c. If result is null, set done to true.
        d. Else result is not null,
           i. Append result to the end of results.
           ii. If global is false, set done to true.
           iii. Else,
                1. Let matchStr be ToString(Get(result, "0")).
                [...]

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let lastIndex be ToLength(Get(R,"lastIndex")).
    [...]
    8. Let sticky be ToBoolean(Get(R, "sticky")).
    [...]
    15. Repeat, while matchSucceeded is false
        [...]
        b. Let r be matcher(S, lastIndex).
        c. If r is failure, then
           i. If sticky is true, then
              [...]
              3. Return null.
features: [Symbol.replace]
---*/

assert.sameValue(/a/yg[Symbol.replace]('aaba', 'x'), 'xxba');
