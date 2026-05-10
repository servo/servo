// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Accumulates consecutive matches when `g` flag is present
es6id: 21.2.5.6
info: |
    21.2.5.6 RegExp.prototype [ @@match ] ( string )

    [...]
    5. Let global be ToBoolean(Get(rx, "global")).
    6. ReturnIfAbrupt(global).
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          ii. ReturnIfAbrupt(result).
          iii. If result is null, then
               1. If n=0, return null.
               2. Else, return A.
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
features: [Symbol.match]
---*/

var result = /a/yg[Symbol.match]('aaba');

assert.notSameValue(result, null);
assert.sameValue(result.length, 2);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'a');
