// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Stops match execution after first match failure
es6id: 21.2.5.6
info: |
    21.2.5.6 RegExp.prototype [ @@match ] ( string )

    [...]
    5. Let global be ToBoolean(Get(rx, "global")).
    6. ReturnIfAbrupt(global).
    7. If global is false, then
       a. Return RegExpExec(rx, S).

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

assert.sameValue(/b/y[Symbol.match]('ab'), null);
