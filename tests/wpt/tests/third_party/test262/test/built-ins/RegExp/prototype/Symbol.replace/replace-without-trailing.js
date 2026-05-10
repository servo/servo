// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Return value when replacement pattern matches final code point
es6id: 21.2.5.8
info: |
    [...]
    17. If nextSourcePosition ≥ lengthS, return accumulatedResult.
features: [Symbol.replace]
---*/

assert.sameValue(/abcd/[Symbol.replace]('abcd', 'X'), 'X');
assert.sameValue(/bcd/[Symbol.replace]('abcd', 'X'), 'aX');
assert.sameValue(/cd/[Symbol.replace]('abcd', 'X'), 'abX');
assert.sameValue(/d/[Symbol.replace]('abcd', 'X'), 'abcX');
