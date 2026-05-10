// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error thrown while setting `lastIndex` after a match failure
es6id: 21.2.5.6
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
    3. Let length be the number of code units in S.
    [...]
    11. Let matchSucceeded be false.
    12. Repeat, while matchSucceeded is false
        a. If lastIndex > length, then
           i. If global is true or sticky is true, then
              1. Perform ? Set(R, "lastIndex", 0, true).
           ii. Return null.
features: [Symbol.match]
---*/

var r = /a/g;
Object.defineProperty(r, 'lastIndex', { writable: false });

assert.throws(TypeError, function() {
  r[Symbol.match]('b');
});
