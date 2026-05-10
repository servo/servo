// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Honors initial value of the `lastIndex` property
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    13. Repeat, while done is false
        a. Let result be RegExpExec(rx, S).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let lastIndex be ToLength(Get(R,"lastIndex")).
    [...]
    8. Let sticky be ToBoolean(Get(R, "sticky")).
    9. ReturnIfAbrupt(sticky).
    10. If global is false and sticky is false, let lastIndex be 0.
features: [Symbol.replace]
---*/

var r = /./y;
r.lastIndex = 1;
assert.sameValue(r[Symbol.replace]('aaa', 'x'), 'axa');
