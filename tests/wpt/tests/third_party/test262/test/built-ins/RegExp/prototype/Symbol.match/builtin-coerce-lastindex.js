// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Type coercion of `lastIndex` property value
es6id: 21.2.5.6
info: |
    [...]
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    7. Return RegExpBuiltinExec(R, S).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    8. Else, let lastIndex be ? ToLength(? Get(R, "lastIndex")).
features: [Symbol.match]
---*/

var r = /./y;
var result;

r.lastIndex = '1.9';

result = r[Symbol.match]('abc');

assert.notSameValue(result, null);
assert.sameValue(result.length, 1);
assert.sameValue(result[0], 'b');
