// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Stops match execution after first match failure
es6id: 21.2.5.9
info: |
    21.2.5.9 RegExp.prototype [ @@search ] ( string )

    [...]
    9. Let result be RegExpExec(rx, S).

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
features: [Symbol.search]
---*/

assert.sameValue(/a/y[Symbol.search]('ba'), -1);
