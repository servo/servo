// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: The `limit` argument is applied to capturing groups
info: |
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        [...]
        c. Let z be RegExpExec(splitter, S).
        [...]
        f. Else z is not null,
           [...]
           iv. Else e ≠ p,
               [...]
               11. Repeat, while i ≤ numberOfCaptures.
                   [...]
                   f. If lengthA = lim, return A.
features: [Symbol.split]
---*/

var result = /c(d)(e)/[Symbol.split]('abcdefg', 2);

assert(Array.isArray(result));
assert.sameValue(result.length, 2);
assert.sameValue(result[0], 'ab');
assert.sameValue(result[1], 'd');
