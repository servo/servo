// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Honors initial value of the `lastIndex` property
es6id: 21.2.5.2
info: |
    21.2.5.2 RegExp.prototype.exec ( string )

    [...]
    6. Return RegExpBuiltinExec(R, S).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let lastIndex be ToLength(Get(R,"lastIndex")).
    [...]
    8. Let sticky be ToBoolean(Get(R, "sticky")).
    9. ReturnIfAbrupt(sticky).
    10. If global is false and sticky is false, let lastIndex be 0.
---*/

var r = /./y;
var match;
r.lastIndex = 1;

match = r.exec('abc');

assert(match !== null);
assert.sameValue(match.length, 1);
assert.sameValue(match[0], 'b');
