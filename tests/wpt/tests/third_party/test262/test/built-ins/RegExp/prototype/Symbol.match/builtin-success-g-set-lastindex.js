// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Setting `lastIndex` after a "global" match success
esid: sec-regexp.prototype-@@match
info: |
    [...]
    5. Let global be ToBoolean(Get(rx, "global")).
    6. ReturnIfAbrupt(global).
    7. If global is false, then
       a. Return RegExpExec(rx, S).

    21.2.5.2.1 Runtime Semantics: RegExpExec ( R, S )

    [...]
    7. Return RegExpBuiltinExec(R, S).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    5. Let flags be the value of R's [[OriginalFlags]] internal slot.
    6. If flags contains "g", let global be true, else let global be false.
    [...]
    16. Let e be r's endIndex value.
    [...]
    18. If global is true or sticky is true,
        a. Let setStatus be Set(R, "lastIndex", e, true).
features: [Symbol.match]
---*/

// The conflicting values for the "global" flag are necessary to observe the
// final modification of `lastIndex` in RegExpBuiltinExec
var r = /b/g;
Object.defineProperty(r, 'global', { value: false });

r[Symbol.match]('abc');

assert.sameValue(r.lastIndex, 2);
