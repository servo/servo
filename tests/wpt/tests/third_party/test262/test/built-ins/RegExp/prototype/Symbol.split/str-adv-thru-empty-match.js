// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    lastIndex is explicitly advanced following an empty match
es6id: 21.2.5.11
info: |
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

var result = /(?:)/[Symbol.split]('abc');

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'b');
assert.sameValue(result[2], 'c');
