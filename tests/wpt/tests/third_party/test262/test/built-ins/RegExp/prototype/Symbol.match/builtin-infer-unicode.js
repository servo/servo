// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Extended unicode support is determined by internal slot (not `unicode`
    property)
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
    12. Let flags be the value of R’s [[OriginalFlags]] internal slot.
    13. If flags contains "u", let fullUnicode be true, else let fullUnicode be
        false.
features: [Symbol.match]
---*/

var r;

r = /\udf06/;
Object.defineProperty(r, 'unicode', { value: true });
assert.notSameValue(r[Symbol.match]('\ud834\udf06'), null);

r = /\udf06/u;
Object.defineProperty(r, 'unicode', { value: false });
assert.sameValue(r[Symbol.match]('\ud834\udf06'), null);
