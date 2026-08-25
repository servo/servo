// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    lastIndex is advanced according to width of astral symbols following match success
es6id: 21.2.5.11
info: |
    21.2.5.11 RegExp.prototype [ @@split ] ( string, limit )

    [...]
    7. Let flags be ToString(Get(rx, "flags")).
    8. ReturnIfAbrupt(flags).
    9. If flags contains "u", let unicodeMatching be true.
    10. Else, let unicodeMatching be false.
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        b. ReturnIfAbrupt(setStatus).
        c. Let z be RegExpExec(splitter, S).
        d. ReturnIfAbrupt(z).
        e. If z is null, let q be AdvanceStringIndex(S, q, unicodeMatching).
        f. Else z is not null,
           i. Let e be ToLength(Get(splitter, "lastIndex")).
           ii. ReturnIfAbrupt(e).
           iii. If e = p, let q be AdvanceStringIndex(S, q, unicodeMatching).
features: [Symbol.split]
---*/

var result = /./u[Symbol.split]('\ud834\udf06');

assert.sameValue(result.length, 2);
assert.sameValue(result[0], '');
assert.sameValue(result[1], '');
