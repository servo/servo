// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Substitution pattern: text before match
es6id: 21.2.5.8
info: |
    16. Repeat, for each result in results,
        [...]
        m. If functionalReplace is true, then
           [...]
        n. Else,
           i. Let replacement be GetSubstitution(matched, S, position,
              captures, replaceValue).
        [...]

    21.1.3.14.1 Runtime Semantics: GetSubstitution

    Code units: 0x0024, 0x0060

    Unicode Characters: $`

    Replacement text:
    If position is 0, the replacement is the empty String. Otherwise the
    replacement is the substring of str that starts at index 0 and whose last
    code unit is at index `position-1`.
features: [Symbol.replace]
---*/

assert.sameValue(/a/[Symbol.replace]('abc', '[$`]'), '[]bc');
assert.sameValue(/b/[Symbol.replace]('abc', '[$`]'), 'a[a]c');
