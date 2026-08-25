// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Successful match of empty string
info: |
    [...]
    22. If size = 0, then
        a. Let z be RegExpExec(splitter, S).
        b. ReturnIfAbrupt(z).
        c. If z is not null, return A.
features: [Symbol.split]
---*/

var result = /(?:)/[Symbol.split]('');

assert(Array.isArray(result));
assert.sameValue(result.length, 0);
