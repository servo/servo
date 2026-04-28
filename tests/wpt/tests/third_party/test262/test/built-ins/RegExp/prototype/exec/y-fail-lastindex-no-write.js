// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Match failure with non-writable `lastIndex` property
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
    [...]
    15. Repeat, while matchSucceeded is false
        [...]
        b. Let r be matcher(S, lastIndex).
        c. If r is failure, then
           i. If sticky is true, then
              1. Let setStatus be Set(R, "lastIndex", 0, true).
              2. ReturnIfAbrupt(setStatus).
---*/

var r = /c/y;
Object.defineProperty(r, 'lastIndex', {
  writable: false
});

assert.throws(TypeError, function() {
  r.exec('abc');
});
