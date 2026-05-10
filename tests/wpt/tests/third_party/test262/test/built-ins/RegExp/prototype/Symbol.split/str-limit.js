// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Results limited to number specified as second argument
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
               3. Perform CreateDataProperty(A, ToString(lengthA), T).
               4. Let lengthA be lengthA +1.
               5. If lengthA = lim, return A.
features: [Symbol.split]
---*/

var result = /x/[Symbol.split]('axbxcxdxe', 3);

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], 'a');
assert.sameValue(result[1], 'b');
assert.sameValue(result[2], 'c');
